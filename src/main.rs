mod loading;
#[macro_use]
mod bind;
mod meshes_tree;
mod rotating;

use std::{iter::zip, sync::{Arc, Weak}};

use bevy::{
    asset::AssetMetaCheck, color::palettes::tailwind::{CYAN_300, GREEN_300, YELLOW_300}, diagnostic::LogDiagnosticsPlugin, ecs::system::SystemId, prelude::*, window::{PresentMode, WindowResized}
};
//use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use loading::{unload_current_visualization, LoadingData, LoadingState, VisualizationComponents};
use bevy_web_asset::WebAssetPlugin;
use meshes_tree::MeshTreeNode;
use rotating::{rotate, Rotate};

#[derive(Resource, Component)]
pub struct MeshTreeRes {
    // not meant to be used, just to keep a strong reference to the root
    _root: Arc<MeshTreeNode>,

    // the current node of the tree to render (which might be a leave with
    // just one mesh or a menu with multiple meshes to select from)
    current: Weak<MeshTreeNode>,

    // some materials
    white_matl: Handle<StandardMaterial>,
    hover_matl: Handle<StandardMaterial>,
    pressed_matl: Handle<StandardMaterial>,
    up_matl: Handle<StandardMaterial>,
}

#[derive(Resource)]
pub struct OneShotSystemsRes {
    update_current_sys: SystemId
}

#[derive(Component)]
pub enum CameraType {
    Rotating,
    Fixed,
}

impl FromWorld for OneShotSystemsRes {
    fn from_world(world: &mut World) -> Self {
        OneShotSystemsRes {
            update_current_sys: world.register_system(update_current_sys)
        }
    }
}

fn main() {
    //def.set(plugin)
    let window = WindowPlugin {
        primary_window: Some(Window {
            present_mode: PresentMode::AutoNoVsync, // Reduces input lag.
            fit_canvas_to_parent: true,
            title: "Cyber Bevy".to_string(),
            ..default()
        }),
        ..default()
    };
    let asset = AssetPlugin {
        meta_check: AssetMetaCheck::Never,
        ..default()
    };

    App::new()
        .add_plugins(WebAssetPlugin::default())
        .add_plugins(DefaultPlugins.set(asset).set(window))
        .add_plugins(bevy_stl::StlPlugin)
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_plugins(PanOrbitCameraPlugin)
        .add_plugins(loading::LoadingScreenPlugin)
        .add_plugins(MeshPickingPlugin)
        //.add_plugins(WorldInspectorPlugin::new())
        //.add_plugins(FrameTimeDiagnosticsPlugin)
        .init_resource::<OneShotSystemsRes>()
        .add_systems(Startup, (unload_current_visualization, setup).chain())
        .add_systems(Update, rotate)
        .add_systems(Update, update_window_size)
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    one_shot_systems: ResMut<OneShotSystemsRes>,
) {
    // light
    let light = commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(1.0, 1.0, 1.0).looking_at(Vec3::ZERO, Vec3::Y),
        VisualizationComponents,
        Visibility::Hidden,
    )).id();

    // camera
    commands.spawn((
        Camera3d::default(),
        PanOrbitCamera::default(),
        Camera {
            is_active: false,
            ..default()
        },
        Transform::default(),
        VisualizationComponents,
    )).add_child(light);

    // materials
    let white_matl = materials.add(Color::WHITE);
    let hover_matl = materials.add(Color::from(CYAN_300));
    let pressed_matl = materials.add(Color::from(YELLOW_300));
    let up_matl = materials.add(Color::from(GREEN_300));

    // load tree of meshes to navigate through
    let mesh_tree_root = MeshTreeNode::from_json(r#"{
        "url": "useless",
        "children": [
            { "url": "http://localhost:8080/benchy.stl" },
            { "url": "http://localhost:8080/mendocino.stl" },
            { "url": "http://localhost:8080/benchy.stl" },
            { "url": "http://localhost:8080/mendocino.stl" },
            { "url": "http://localhost:8080/benchy.stl" },
            { "url": "http://localhost:8080/mendocino.stl" },
            { "url": "https://files.printables.com/media/prints/1109750/stls/8384921_616f48b1-343a-44be-9706-270e717d37fc_c57a5d1a-11f4-419d-b7ed-69836936406b/notredameparis2.stl" },
            { "url": "https://files.printables.com/media/prints/888961/stls/6807211_1d4002d8-8af3-41bb-8706-1a0fa0cfd25b_52419fc3-bee6-4483-8843-fb1590d37704/einsteinpot.stl" },
            { "url": "https://files.printables.com/media/prints/2236/stls/14012_b9139bd5-c68b-46a5-ba28-6513f9715d83/3dbenchy.stl" }
        ]
    }"#);
    console_log!("Meshes: {mesh_tree_root:?}");
    let initial_mesh_node = Arc::downgrade(&mesh_tree_root);

    // setup the main resource
    commands.insert_resource(
        MeshTreeRes {
            _root: mesh_tree_root,
            current: initial_mesh_node,
            white_matl,
            hover_matl,
            pressed_matl,
            up_matl,
        }
    );

    // show the initial entities
    commands.run_system(one_shot_systems.update_current_sys);
}

fn update_current_sys(
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
    mut loading_data: ResMut<LoadingData>,
    mesh_tree: Res<MeshTreeRes>,
    current_meshes: Query<Entity, With<Mesh3d>>,
    mut camera: Query<&mut PanOrbitCamera, With<Camera3d>>,
    window: Query<&Window>,
) {
    console_log!("update_current_sys called");

    // despawn all current meshes
    current_meshes.iter().for_each(|entity| commands.entity(entity).despawn());

    let Some(mesh_tree_node) = mesh_tree.current.upgrade() else {
        // something went wrong, nothing to do
        return;
    };

    // switch to the loading state, since we are going to load more assets
    commands.set_state(LoadingState::Loading);
    let mut camera_pan_orbit = camera.single_mut();
    let window = window.single();

    match get_render_mode(&mesh_tree_node) {
        MeshRenderMode::Leaf { url } => {
            // we need to render a single item and let the user move the camera
            camera_pan_orbit.enabled = true;
            camera_pan_orbit.target_radius = 2.0;
            camera_pan_orbit.target_yaw = 0.5;
            camera_pan_orbit.target_pitch = 0.5;

            let model = asset_server.load(url);
            loading_data.add_asset(&model);
            commands.spawn((
                Mesh3d(model),
                MeshMaterial3d(mesh_tree.white_matl.clone()),
                Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)).with_scale(Vec3::splat(0.02)),
                VisualizationComponents,
                Visibility::Hidden,
            ));
        },

        MeshRenderMode::Subtree { urls } => {
            // we need to render multiple rotating items but the camera should stay still
            camera_pan_orbit.enabled = false;
            camera_pan_orbit.target_radius = 2.0;
            camera_pan_orbit.target_yaw = 0.0;
            camera_pan_orbit.target_pitch = 0.0;

            let (positions, scale) = generate_positions(urls.len(), window.height(), window.width());
            console_log!("positions {positions:?}, scale {scale}");
            for ((child_index, url), (h, w)) in zip(urls, positions) {
                let model = asset_server.load(url);
                loading_data.add_asset(&model);
                commands.spawn((
                    Mesh3d(model),
                    MeshMaterial3d(mesh_tree.white_matl.clone()),
                    Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
                        .with_scale(Vec3::splat(scale * 0.02))
                        .with_translation(Vec3 { x: w, y: h, z: 0.0 }),
                    VisualizationComponents,
                    Visibility::Hidden,
                    Rotate,
                ))
                    .observe(update_material_on::<Pointer<Over>>(mesh_tree.hover_matl.clone()))
                    .observe(update_material_on::<Pointer<Out>>(mesh_tree.white_matl.clone()))
                    .observe(update_material_on::<Pointer<Down>>(mesh_tree.pressed_matl.clone()))
                    .observe(child_child_as_current_on::<Pointer<Up>>(child_index));
            }
        },
    }
}

enum MeshRenderMode {
    Leaf { url: String },
    Subtree { urls: Vec<(usize, String)> },
}

fn get_render_mode(mesh_tree_node: &Arc<MeshTreeNode>) -> MeshRenderMode {
    console_log!("get_render_mode {mesh_tree_node:?}");

    if mesh_tree_node.children.is_empty() {
        return MeshRenderMode::Leaf { url: mesh_tree_node.url.clone() };
    }

    if mesh_tree_node.children.len() == 1 {
        if let Some(child) = mesh_tree_node.children.first() {
            if child.children.is_empty() {
                return MeshRenderMode::Leaf { url: child.url.clone() }
            }
        }
    }

    MeshRenderMode::Subtree {
        urls: mesh_tree_node.children.iter()
            .enumerate()
            .map(|(index, child)| (index, child.url.clone()))
            .collect()
    }
}

fn update_window_size(
    mut commands: Commands,
    mut events: EventReader<WindowResized>,
    mesh_tree: Res<MeshTreeRes>,
    one_shot_systems: ResMut<OneShotSystemsRes>,
) {
    for window_size in events.read() {
        // if the window size changed, relayout the meshes
        console_log!("window size changed to {window_size:?}");
        if let Some(MeshRenderMode::Subtree { .. }) = mesh_tree.current.upgrade().map(|n| get_render_mode(&n)) {
            commands.run_system(one_shot_systems.update_current_sys);
        }
    }
}

fn generate_positions(mesh_count: usize, window_height: f32, window_width: f32) -> (Vec<(f32, f32)>, f32) {
    let ratio = window_width / window_height;
    let height = (mesh_count as f32 / ratio).sqrt();
    let width = (mesh_count as f32 * ratio).sqrt();
    let viewport_height = 1.0; // the Bevy viewport has a fixed height
    let viewport_width = viewport_height * ratio;

    let (height, width) = if height.ceil() - height < width.ceil() - width {
        let height = height.ceil() as usize;
        (height, mesh_count.div_ceil(height))
    } else {
        let width = width.ceil() as usize;
        (mesh_count.div_ceil(width), width)
    };
    assert!(height * width >= mesh_count);

    let mut positions = Vec::new();
    for i in 0..height {
        for j in 0..width {
            let (i, j, height, width) = (i as f32, j as f32, height as f32, width as f32);
            positions.push((
                viewport_height / 2.0 - viewport_height / height * (i + 0.5),
                -viewport_width / 2.0 + viewport_width / width * (j + 0.5),
            ));
        }
    }
    positions.truncate(mesh_count);

    let scale = f32::min(viewport_height / (height as f32), viewport_width / (width as f32));
    (positions, scale)
}

/// Returns an observer that updates the entity's material to the one specified.
fn update_material_on<E>(
    new_material: Handle<StandardMaterial>,
) -> impl Fn(Trigger<E>, Query<&mut MeshMaterial3d<StandardMaterial>>) {
    // An observer closure that captures `new_material`. We do this to avoid needing to write four
    // versions of this observer, each triggered by a different event and with a different hardcoded
    // material. Instead, the event type is a generic, and the material is passed in.
    move |trigger, mut query| {
        if let Ok(mut material) = query.get_mut(trigger.entity()) {
            material.0 = new_material.clone();
        }
    }
}

fn child_child_as_current_on<E>(
    child_index: usize
) -> impl Fn(Trigger<E>, Commands, ResMut<MeshTreeRes>, ResMut<OneShotSystemsRes>) {
    move |_, mut commands, mut mesh_tree, one_shot_systems| {
        let Some(current) = mesh_tree.current.upgrade() else { return; };
        let Some(child) = current.children.get(child_index) else { return; };
        mesh_tree.current = Arc::downgrade(child);
        commands.run_system(one_shot_systems.update_current_sys);
    }
}

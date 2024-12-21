//! A simple 3D scene with light shining over a cube sitting on a plane.

mod loading;
#[macro_use]
mod bind;
mod meshes_tree;

use std::sync::{Arc, Weak};

use bevy::{
    asset::AssetMetaCheck, color::palettes::tailwind::{CYAN_300, YELLOW_300}, diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin}, prelude::*, render::mesh, window::PresentMode
};
//use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use loading::{unload_current_visualization, LoadingData, VisualizzationComponents};
use bevy_web_asset::WebAssetPlugin;
use meshes_tree::MeshTreeNode;

#[derive(Default, Debug, Resource)]
pub enum Resolution {
    #[default]
    Cube,
}

#[derive(Resource, Component)]
pub struct MeshTreeRes {
    root: Arc<MeshTreeNode>,
    current: Weak<MeshTreeNode>,
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
        .add_plugins(MeshPickingPlugin)
        //.add_plugins(WorldInspectorPlugin::new())
        .init_resource::<Resolution>()
        .add_plugins(bevy_stl::StlPlugin)
        .add_systems(
            Update,
            (unload_current_visualization, setup).chain().run_if(resource_changed::<Resolution>),
        )
        .add_plugins(FrameTimeDiagnosticsPlugin)
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_plugins(PanOrbitCameraPlugin)
        .add_plugins(loading::LoadingScreenPlugin)
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: ResMut<AssetServer>,
    mut loading_data: ResMut<LoadingData>,
) {
    // Set up the materials.
    let white_matl = materials.add(Color::WHITE);
    let hover_matl = materials.add(Color::from(CYAN_300));
    let pressed_matl = materials.add(Color::from(YELLOW_300));

    let mesh_tree_root = MeshTreeNode::from_json(r#"{
        "url": "http://localhost:8080/mendocino.stl",
        "children": [
            {
                "url": "http://localhost:8080/benchy.stl"
            }
        ]
    }"#);
    console_log!("Meshes: {mesh_tree_root:?}");
    let initial_mesh_node = Arc::downgrade(&mesh_tree_root);
    commands.insert_resource(MeshTreeRes { root: mesh_tree_root, current: initial_mesh_node });

    let model = asset_server.load(bind::get_url_fragment());
    loading_data.add_asset(&model);
    commands.spawn((
        Mesh3d(model),
        MeshMaterial3d(white_matl.clone()),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)).with_scale(Vec3::splat(0.05)),
        VisualizzationComponents,
        Visibility::Hidden,
    ))
        .observe(update_material_on::<Pointer<Over>>(hover_matl.clone()))
        .observe(update_material_on::<Pointer<Out>>(white_matl.clone()))
        .observe(update_material_on::<Pointer<Down>>(pressed_matl.clone()))
        .observe(update_material_on::<Pointer<Up>>(hover_matl.clone()));

    // light
    let light = commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(1.0, 1.0, 1.0).looking_at(Vec3::ZERO, Vec3::Y),
        VisualizzationComponents,
        Visibility::Hidden,
    )).id();

    // camera
    commands.spawn((
        Camera3d::default(),
        PanOrbitCamera {
            pitch_lower_limit: Some(0.0),
            ..default()
        },
        Camera {
            is_active: false,
            ..default()
        },
        Transform::from_xyz(6.0, 7.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
        VisualizzationComponents,
    )).add_child(light);
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


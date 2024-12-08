//! A simple 3D scene with light shining over a cube sitting on a plane.

mod loading;

use bevy::{
    asset::AssetMetaCheck, diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin}, prelude::*
};
//use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use loading::{unload_current_visualization, LoadingData, VisualizzationComponents};
use bevy_web_asset::WebAssetPlugin;
#[derive(Default, Debug, Resource)]
pub enum Resolution {
    #[default]
    Cube,
}

fn main() {
    //def.set(plugin)
    let window = WindowPlugin {
        primary_window: Some(Window {
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
        .add_plugins((WebAssetPlugin::default(), DefaultPlugins.set(asset).set(window)))
        //.add_plugins(WorldInspectorPlugin::new())
        .init_resource::<Resolution>()
        .add_plugins(bevy_stl::StlPlugin)
        .add_systems(
            Update,
            (unload_current_visualization, setup, ).chain().run_if(resource_changed::<Resolution>),
        )
        .add_plugins((FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin::default()))
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
    let model = asset_server.load("http://localhost:8080/benchy.stl");
    loading_data.add_asset(&model);
    commands.spawn((
        Mesh3d(model),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)).with_scale(Vec3::splat(0.05)),
        VisualizzationComponents,
        Visibility::Hidden,
    ));

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

//! Shows how to create a loading screen that waits for assets to load and render.

use bevy::prelude::*;
use pipelines_ready::*;

// The way we'll go about doing this in this example is to
// keep track of all assets that we want to have loaded before
// we transition to the desired scene.
//
// In order to ensure that visual assets are fully rendered
// before transitioning to the scene, we need to get the
// current status of cached pipelines.
//
// While loading and pipelines compilation is happening, we
// will show a loading screen. Once loading is complete, we
// will transition to the scene we just loaded.

pub struct LoadingScreenPlugin;

fn is_loading(r: Res<LoadingState>) -> bool {
    *r == LoadingState::Loading
}

impl Plugin for LoadingScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PipelinesReadyPlugin)
            .insert_resource(LoadingState::default())
            .insert_resource(LoadingData::new(5))
            .add_systems(Startup, setup)
            //load_loading_screen
            .add_systems(
                Update,
                (update_loading_data, update_loading_screen).run_if(is_loading),
            )
            .add_systems(
                Update,
                clear_loading_screen.run_if(resource_changed::<LoadingState>),
            );
    }
}

fn clear_loading_screen(
    mut commands: Commands,
    res: Res<LoadingState>,
    loading: Query<Entity, With<LoadingScreen>>,
    mut loaded: Query<(Entity, &mut Visibility), With<VisualizzationComponents>>,
    mut camera: Option<Single<&mut Camera, With<VisualizzationComponents>>>,
    loading_data: Res<LoadingData>,
) {
    match res.as_ref() {
        LoadingState::Ready => {
            for entity in loading.iter() {
                commands.entity(entity).despawn_recursive();
            }
            loaded.iter_mut().for_each(|(_, mut visibility)| {
                *visibility = Visibility::Visible;
            });
            if let Some(camera) = camera.as_mut() {
                camera.is_active = true;
            }
        }
        LoadingState::Loading => {
            for (entity, _) in loaded.iter() {
                commands.entity(entity).despawn_recursive();
            }
            commands.spawn((
                LoadingScreen,
                Sprite {
                    image: loading_data.img.clone(),
                    ..default()
                },
            ));
            commands.spawn((LoadingScreen, Camera2d));
        }
    }
}

// A `Resource` that holds the current loading state.
#[derive(Resource, Default, PartialEq, Eq)]
pub enum LoadingState {
    #[default]
    Ready,
    Loading,
}

// A resource that holds the current loading data.
#[derive(Resource, Debug, Default)]
pub struct LoadingData {
    img: Handle<Image>,
    // This will hold the currently unloaded/loading assets.
    loading_assets: Vec<UntypedHandle>,
    // Number of frames that everything needs to be ready for.
    // This is to prevent going into the fully loaded state in instances
    // where there might be a some frames between certain loading/pipelines action.
    confirmation_frames_target: usize,
    // Current number of confirmation frames.
    confirmation_frames_count: usize,
}
impl LoadingData {
    #[allow(dead_code)]
    pub fn add_asset<C: Asset>(&mut self, asset: &Handle<C>) {
        self.loading_assets.push(asset.clone().untyped());
    }
}

impl LoadingData {
    fn new(confirmation_frames_target: usize) -> Self {
        Self {
            img: Handle::default(),
            loading_assets: Vec::new(),
            confirmation_frames_target,
            confirmation_frames_count: 0,
        }
    }
}

// This resource will hold the level related systems ID for later use.
fn setup(asset_server: ResMut<AssetServer>, mut loading_data: ResMut<LoadingData>) {
    loading_data.img = asset_server.load("http://localhost:8080/logo.png");
}

// Marker component for easier deletion of entities.
#[derive(Component)]
#[require(Visibility(|| Visibility::Visible))]
pub struct VisualizzationComponents;

// Removes all currently loaded level assets from the game World.
pub fn unload_current_visualization(mut loading_state: ResMut<LoadingState>) {
    *loading_state = LoadingState::Loading;
}

// Monitors current loading status of assets.
fn update_loading_data(
    mut loading_data: ResMut<LoadingData>,
    mut loading_state: ResMut<LoadingState>,
    asset_server: Res<AssetServer>,
    pipelines_ready: Res<PipelinesReady>,
) {
    if !loading_data.loading_assets.is_empty() || !pipelines_ready.0 {
        // If we are still loading assets / pipelines are not fully compiled,
        // we reset the confirmation frame count.
        loading_data.confirmation_frames_count = 0;

        // Go through each asset and verify their load states.
        // Any assets that are loaded are then added to the pop list for later removal.
        let mut pop_list: Vec<usize> = Vec::new();
        for (index, asset) in loading_data.loading_assets.iter().enumerate() {
            if let Some(state) = asset_server.get_load_states(asset) {
                if state.2.is_loaded() {
                    pop_list.push(index);
                }
            }
        }

        // Remove all loaded assets from the loading_assets list.
        for i in pop_list.iter() {
            loading_data.loading_assets.remove(*i);
        }

        // If there are no more assets being monitored, and pipelines
        // are compiled, then start counting confirmation frames.
        // Once enough confirmations have passed, everything will be
        // considered to be fully loaded.
    } else {
        loading_data.confirmation_frames_count += 1;
        if loading_data.confirmation_frames_count == loading_data.confirmation_frames_target {
            *loading_state = LoadingState::Ready;
        }
    }
}

// Marker tag for loading screen components.
#[derive(Component)]
struct LoadingScreen;

// Determines when to show the loading screen
fn update_loading_screen(
    mut image: Query<&mut Transform, (With<LoadingScreen>, With<Sprite>)>,
    timer: Res<Time>,
) {
    image
        .iter_mut()
        .for_each(|mut x| x.rotate_z(-timer.delta_secs()));
}

mod pipelines_ready {
    use bevy::{
        prelude::*,
        render::{render_resource::*, *},
    };

    pub struct PipelinesReadyPlugin;
    impl Plugin for PipelinesReadyPlugin {
        fn build(&self, app: &mut App) {
            app.insert_resource(PipelinesReady::default());

            // In order to gain access to the pipelines status, we have to
            // go into the `RenderApp`, grab the resource from the main App
            // and then update the pipelines status from there.
            // Writing between these Apps can only be done through the
            // `ExtractSchedule`.
            app.sub_app_mut(RenderApp)
                .add_systems(ExtractSchedule, update_pipelines_ready);
        }
    }

    #[derive(Resource, Debug, Default)]
    pub struct PipelinesReady(pub bool);

    fn update_pipelines_ready(mut main_world: ResMut<MainWorld>, pipelines: Res<PipelineCache>) {
        if let Some(mut pipelines_ready) = main_world.get_resource_mut::<PipelinesReady>() {
            pipelines_ready.0 = pipelines.waiting_pipelines().count() == 0;
        }
    }
}

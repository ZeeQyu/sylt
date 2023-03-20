use bevy::asset::LoadState;
use crate::imports::*;
use bevy_asset_loader::prelude::*;
use bevy_yoleck::YoleckLevelIndex;

#[derive(Default)]
pub struct LevelsPlugin;

impl Plugin for LevelsPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(LevelInformation { current_index: 0 })
            .insert_resource(LoadingObjects::default())
            .add_startup_system(setup)
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::LoadLevels)
                    .with_system(handle_loading)
                    .into()
            );
        ;
    }
}

#[derive(Resource)]
struct LevelInformation {
    current_index: usize,
}

#[derive(Resource, Deref, DerefMut, Default)]
struct LoadingObjects {
    level_index: Option<Handle<YoleckLevelIndex>>,
}

fn setup(asset_server: Res<AssetServer>, mut loading: ResMut<LoadingObjects>) {
    loading.level_index = Some(asset_server.load("levels/index.yoli") as Handle<YoleckLevelIndex>);
}

fn handle_loading(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut loading: ResMut<LoadingObjects>,
    assets: Res<Assets<YoleckLevelIndex>>,
) {
    println!("Checking level indexd");
    if let Some(handle) = &loading.level_index {
        match asset_server.get_load_state(handle) {
            LoadState::Loaded => {
                println!("{:?}", assets.get(handle));
                commands.insert_resource(NextState(GameState::Game));
            }
            _ => ()
        }
    }
}

pub(crate) fn start_first_level(
    asset_server: Res<AssetServer>,
    mut yoleck_loading_command: ResMut<bevy_yoleck::YoleckLoadingCommand>,
) {
    *yoleck_loading_command = bevy_yoleck::YoleckLoadingCommand::FromAsset(
        asset_server.load(std::path::Path::new("levels").join("Two clusters.yol")),
    );
}

fn start_next_level(
    asset_server: Res<AssetServer>,
    level_information: Res<LevelInformation>,
    configuration: Res<Configuration>,
) {
    // configuration.global_assets.level_index;
}

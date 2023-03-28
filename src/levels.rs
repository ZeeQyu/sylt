use bevy::asset::LoadState;
use crate::imports::*;
use bevy_yoleck::{YoleckLevelIndex, YoleckRawLevel};

#[derive(Default)]
pub struct LevelsPlugin;

impl Plugin for LevelsPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(LevelInformation::default())
            .add_event::<LevelEvent>()
            .add_startup_system(setup)
            .add_system(wait_for_level_index.run_in_state(GameState::LoadLevelIndex))
            .add_system(wait_for_levels.run_in_state(GameState::LoadLevels))
            .add_system(handle_level_events)
        ;
    }
}

#[derive(Resource, Default)]
pub struct LevelInformation {
    current_index: usize,
    level_index: Option<Handle<YoleckLevelIndex>>,
    levels: Vec<Handle<YoleckRawLevel>>,
}

pub enum LevelEvent {
    LoadLevelIndex { index: usize },
    LoadNextLevel,
}

fn setup(
    asset_server: Res<AssetServer>,
    mut level_information: ResMut<LevelInformation>,
) {
    level_information.level_index = Some(asset_server.load("levels/index.yoli") as Handle<YoleckLevelIndex>);
}

fn wait_for_level_index(
    mut commands: Commands,
    mut level_information: ResMut<LevelInformation>,
    asset_server: Res<AssetServer>,
    assets: Res<Assets<YoleckLevelIndex>>,
) {
    println!("Checking level index");
    let handle = level_information.level_index.as_ref().expect("We should have an index at this point.");
    match asset_server.get_load_state(handle) {
        LoadState::Loaded => {
            println!("{:?}", assets.get(handle));
            if let Some(level_index) = assets.get(handle) {
                assert!(level_index.len() > 0);
                let mut levels = vec![];
                for entry in level_index.iter() {
                    let level = asset_server.load(format!("levels/{}", entry.filename));
                    levels.push(level);
                }
                level_information.levels = levels;
                level_information.level_index = None;
                commands.insert_resource(NextState(GameState::LoadLevels));
            }
        }
        _ => {}
    }
}

fn wait_for_levels(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut level_information: ResMut<LevelInformation>,
    mut yoleck_loading_command: ResMut<bevy_yoleck::YoleckLoadingCommand>,
) {
    println!("Checking levels");
    assert!(level_information.levels.len() > 0);
    level_information.levels.iter().all(|level| -> bool {
        asset_server.get_load_state(level) == LoadState::Loaded
    }).then(|| {
        level_information.current_index = 0;
        let first_level = dbg!(&level_information.levels).get(level_information.current_index)
            .expect("The first level should be loaded at this point");
        *yoleck_loading_command = bevy_yoleck::YoleckLoadingCommand::FromAsset(first_level.clone());
        commands.insert_resource(NextState(GameState::Game));
    });
}

pub fn handle_level_events(
    mut commands: Commands,
    mut level_information: ResMut<LevelInformation>,
    mut event_reader: EventReader<LevelEvent>,
    mut yoleck_loading_command: ResMut<bevy_yoleck::YoleckLoadingCommand>,
    level_entities_query: Query<Entity, With<bevy_yoleck::YoleckManaged>>,
) {
    for event in event_reader.iter() {
        match event {
            LevelEvent::LoadLevelIndex { index } => {
                for entity in level_entities_query.iter() {
                    commands.entity(entity).despawn_recursive();
                }
                if let Some(level) = level_information.levels.get(*index) {
                    *yoleck_loading_command = bevy_yoleck::YoleckLoadingCommand::FromAsset(level.clone());
                }
            }
            LevelEvent::LoadNextLevel => {
                for entity in level_entities_query.iter() {
                    commands.entity(entity).despawn_recursive();
                }
                level_information.current_index += 1;
                if let Some(first_level) = level_information.levels.get(level_information.current_index) {
                    *yoleck_loading_command = bevy_yoleck::YoleckLoadingCommand::FromAsset(first_level.clone());
                }
            }
        }
    }
}

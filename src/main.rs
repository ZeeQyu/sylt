mod spawning;
mod motion;
mod animation;
mod editor;

use std::time::Duration;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use crate::motion::{apply_player_input, calculate_flocking, calculate_grazing, calculate_inertia, calculate_velocity, Configuration, draw_debug_lines, find_flocking_neighbours, reset_influences, run_from_players};
use bevy_prototype_debug_lines::*;
use bevy_inspector_egui::quick as inspector_egui;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use iyes_loopless::prelude::*;

const TIME_STEP: f32 = 1.0 / 60.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                window: WindowDescriptor {
                    title: String::from("Sylt"),
                    ..default()
                },
                ..default()
            }))
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(30.0))
        .add_plugin(bevy_yoleck::bevy_egui::EguiPlugin)
        .add_plugin(inspector_egui::WorldInspectorPlugin)
        .add_plugin(DebugLinesPlugin::default())
        .add_plugin(inspector_egui::ResourceInspectorPlugin::<Configuration>::default())
        // .add_fixed_timestep(Duration::from_millis(16),
        //                     "my_fixed_update")
        // .add_plugin(motion::MotionPlugin::default())
        // .add_plugin(animation::AnimationPlugin::default())
        .add_plugin(editor::EditorPlugin::default())
        .add_system_set(
            SystemSet::new()
                .label("animation")
                .after("motion")
                // .with_run_criteria(FixedTimestep::step(TIME_STEP as f64))
                .with_system(animation::animate_sprite)
        )
        .add_system_set(
            ConditionSet::new()
                .run_in_state(GameState::Game)
                .label("motion_prep")
                .with_system(find_flocking_neighbours)
                // .with_run_criteria(FixedTimestep::step(TIME_STEP as f64))
                .with_system(reset_influences)
                .into()
        )
        .add_system_set(
            ConditionSet::new()
                .run_in_state(GameState::Game)
                .label("motion")
                .after("motion_prep")
                // .with_run_criteria(FixedTimestep::step(TIME_STEP as f64))
                .with_system(apply_player_input)
                .with_system(run_from_players)
                .with_system(calculate_flocking)
                .with_system(calculate_grazing)
                .with_system(calculate_inertia)
                .into()
        )
        .add_system_set(
            ConditionSet::new()
                .run_in_state(GameState::Game)
                .label("motion_apply")
                .after("motion")
                // .with_run_criteria(FixedTimestep::step(TIME_STEP as f64))
                .with_system(calculate_velocity)
                .with_system(draw_debug_lines)
                .into()
        )

        .add_plugin(RapierDebugRenderPlugin::default())
        // .add_plugin(LogDiagnosticsPlugin::default())
        // .add_plugin(FrameTimeDiagnosticsPlugin::default())

        .register_type::<Configuration>()
        .insert_resource::<Configuration>(Configuration::new())

        .insert_resource(ClearColor(Color::rgb_u8(46 as u8, 34 as u8, 47 as u8)))
        .insert_resource(RapierConfiguration { gravity: Vec2::ZERO, ..default() })
        .add_system(update_zoom)
        .add_startup_system(spawning::setup)
        .run();
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum GameState {
    Game,
    Editor,
}


#[derive(Component)]
pub enum ConfigurationSetId {
    Player,
    Sheep,
    Grass,
}


fn update_zoom(
    mut query: Query<&mut OrthographicProjection>,
    config: Res<Configuration>,
) {
    for mut projection in query.iter_mut() {
        projection.scale = config.zoom;
    }
}

mod motion;
mod animation;
mod editor;
mod entities;
mod imports;
mod configuration;
mod assets;
mod levels;

use bevy_inspector_egui::quick as inspector_egui;
use imports::*;

// const TIME_STEP: f32 = 1.0 / 60.0;

fn main() {
    let is_editor = std::env::args().any(|arg| arg == "--editor");
    let mut app = App::new();
    app.add_plugins(DefaultPlugins
        .set(ImagePlugin::default_nearest())
        .set(WindowPlugin {
            window: WindowDescriptor {
                title: String::from("Sylt"),
                width: 1600.0,
                height: 1000.0,
                ..default()
            },
            ..default()
        }));
    if is_editor {
        app
            .add_loopless_state(GameState::Editor)
            .add_plugin(EditorPlugin::default());
    } else {
        app
            .add_loopless_state(GameState::LoadLevelIndex)
            .add_plugin(bevy_yoleck::YoleckPluginForGame)
            .add_system(game_rules::handle_game_rules.run_in_state(GameState::Game));
    }
    app
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(30.0))
        .add_plugin(bevy_yoleck::bevy_egui::EguiPlugin)
        .add_plugin(DebugLinesPlugin::default())
        .add_plugin(inspector_egui::ResourceInspectorPlugin::<Configuration>::default())
        // .add_plugin(inspector_egui::WorldInspectorPlugin)
        // .add_plugin(RapierDebugRenderPlugin::default())
        // .add_plugin(LogDiagnosticsPlugin::default())
        // .add_plugin(FrameTimeDiagnosticsPlugin::default())
        // .add_plugin(assets::GameAssetPlugin::default())
        .add_plugin(animation::AnimationPlugin::default()) // Needs to be before anything that spawns entities
        .add_plugin(MotionPlugin::default())
        .add_plugin(player::PlayerPlugin::default())
        .add_plugin(sheep::SheepPlugin::default())
        .add_plugin(sheep_cluster::SheepClusterPlugin::default())
        .add_plugin(food::FoodPlugin::default())
        .add_plugin(fence::FencePlugin::default())
        .add_plugin(grass::GrassPlugin::default())
        .add_plugin(text::TextPlugin::default())
        .add_plugin(zone::ZonePlugin::default())
        .add_plugin(levels::LevelsPlugin::default())
        .add_plugin(game_rules::GameRulesPlugin::default())

        .register_type::<Configuration>()
        .insert_resource::<Configuration>(Configuration::new())

        .insert_resource(ClearColor(Color::rgb_u8(46 as u8, 34 as u8, 47 as u8)))
        .insert_resource(RapierConfiguration { gravity: Vec2::ZERO, ..default() })
        .add_system(update_zoom)
        .add_startup_system(spawn_camera);

    app.run();
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum GameState {
    LoadLevelIndex,
    LoadLevels,
    Game,
    Editor,
}


pub fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn update_zoom(
    mut query: Query<&mut OrthographicProjection>,
    config: Res<Configuration>,
) {
    for mut projection in query.iter_mut() {
        projection.scale = config.zoom;
    }
}

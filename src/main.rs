mod spawning;
mod motion;
mod animation;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use crate::motion::Configuration;
use bevy_prototype_debug_lines::*;
use bevy_inspector_egui::quick as inspector_egui;

const TIME_STEP: f32 = 1.0 / 60.0;

fn main() {
    let background_color = Color::rgb_u8(46 as u8, 34 as u8, 47 as u8);
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(30.0))
        // .add_plugin(inspector_egui::WorldInspectorPlugin)
        .add_plugin(DebugLinesPlugin::default())
        .add_plugin(inspector_egui::ResourceInspectorPlugin::<Configuration>::default())
        .add_plugin(motion::MotionPlugin::default())
        .add_plugin(animation::AnimationPlugin::default())

        // .add_plugin(RapierDebugRenderPlugin::default())

        .register_type::<Configuration>()
        .insert_resource::<Configuration>(Configuration::new())

        .insert_resource(ClearColor(background_color))
        .insert_resource(RapierConfiguration { gravity: Vec2::ZERO, ..default() })
        .add_startup_system(spawning::setup)
        .run();
}

#[derive(Component)]
pub enum ConfigurationSetId {
    Player,
    Sheep,
}


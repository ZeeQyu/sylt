use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use rand_distr::{Normal, Distribution};
use crate::{animation, ConfigurationSetId};
use crate::motion::{Configuration, Flocking, Grazing, Inertia, Influences, PlayerInput, RunsFromPlayer};

pub fn setup(
    mut commands: Commands,
    mut config: ResMut<Configuration>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(Camera2dBundle::default());

    animation::load_sprite_sheets(asset_server, &mut texture_atlases, &mut config.animation);
    let normal = Normal::new(0.0, 150.0).unwrap();
    spawn_player(&mut commands, &config.animation.player,Vec3::new(-400.0, 0.0, 0.0));
    for _ in 0..40 {
        let x = normal.sample(&mut rand::thread_rng());
        let y = normal.sample(&mut rand::thread_rng());
        spawn_sheep(&mut commands, &config.animation.sheep, Vec3::new(x, y, 0.0));
    }
}

#[derive(Bundle)]
struct Common {
    animation_bundle: animation::AnimationBundle,
    collider: Collider,
    rigid_body: RigidBody,
    locked_axes: LockedAxes,
    velocity: Velocity,
    influences: Influences,
}

fn spawn_common(config_set: &animation::AnimationSet, position: Vec3) -> Common {
    Common {
        animation_bundle: animation::AnimationBundle::from(config_set, position),
        collider: Collider::ball(15.0),
        rigid_body: RigidBody::Dynamic,
        locked_axes: LockedAxes::ROTATION_LOCKED,
        velocity: Velocity::default(),
        influences: Influences::default(),
    }
}

fn spawn_player(
    commands: &mut Commands,
    config_set: &animation::AnimationSet,
    position: Vec3,
) {
    commands.spawn((
        spawn_common(config_set, position),
        PlayerInput {},
        Dominance::group(10),
        Name::new("Player"),
        ConfigurationSetId::Player,
    ));
}

fn spawn_sheep(
    commands: &mut Commands,
    config_set: &animation::AnimationSet,
    position: Vec3,
) {
    commands.spawn(
        (
            spawn_common(config_set, position),
            Flocking::default(),
            Grazing {
                current_direction: None,
                time_left: (rand::random::<f32>() * 5.0) + 0.0,
            },
            RunsFromPlayer {
                direction: Vec3::ZERO,
                magnitude: 0.0,
            },
            Name::new("Sheep"),
            ConfigurationSetId::Sheep,
            Inertia::default(),
        )
    );
}


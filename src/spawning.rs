use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use rand_distr::{Uniform, Distribution};
use crate::{animation, ConfigurationSetId};
use crate::animation::AnimationBundle;
use crate::motion::{Configuration, Flocking, Grazing, Inertia, Influences, PlayerInput, Runner};
use serde::Serialize;
use serde::Deserialize;

pub fn setup(
    mut commands: Commands,
    mut config: ResMut<Configuration>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(Camera2dBundle::default());

    animation::load_sprite_sheets(asset_server, &mut texture_atlases, &mut config.animation);

    let distribution = Uniform::new(-1000.0, 1000.0);
    for _ in 0..300 {
        let x = distribution.sample(&mut rand::thread_rng());
        let y = distribution.sample(&mut rand::thread_rng());
        commands.spawn(GrassBundle::new(&config.animation.grass, Vec3::new(x, y, 0.0)));
    }
}

#[derive(Bundle)]
pub struct Actor {
    animation_bundle: animation::AnimationBundle,
    collider: Collider,
    rigid_body: RigidBody,
    locked_axes: LockedAxes,
    velocity: Velocity,
    influences: Influences,
}

impl Actor {
    fn new(config_set: &animation::AnimationSheet, position: Vec3, collider: Collider) -> Self {
        Actor {
            animation_bundle: animation::AnimationBundle::from(config_set, position),
            //collider: Collider::ball(15.0),
            rigid_body: RigidBody::Dynamic,
            locked_axes: LockedAxes::ROTATION_LOCKED,
            velocity: Velocity::default(),
            influences: Influences::default(),
            collider,
        }
    }
}

#[derive(Bundle)]
pub struct PlayerBundle {
    actor: Actor,
    player: PlayerInput,
    name: Name,
    config_set_id: ConfigurationSetId,
}

impl PlayerBundle {
    pub fn new(config_set: &animation::AnimationSheet, position: Vec3) -> Self {
        PlayerBundle {
            actor: Actor::new(config_set, position, Collider::ball(15.0)),
            player: PlayerInput {},
            name: Name::new("Player"),
            config_set_id: ConfigurationSetId::Player,
        }
    }
}

#[derive(Bundle)]
pub struct SheepBundle {
    actor: Actor,
    flocking: Flocking,
    grazing: Grazing,
    runner: Runner,
    name: Name,
    config_set_id: ConfigurationSetId,
    inertia: Inertia,
}

impl SheepBundle {
    pub fn new(config_set: &animation::AnimationSheet, position: Vec3) -> Self {
        SheepBundle {
            actor: Actor::new(config_set, position, Collider::ball(13.0)),
            flocking: Flocking::default(),
            grazing: Grazing {
                current_direction: None,
                time_left: (rand::random::<f32>() * 5.0) + 0.0,
            },
            runner: Runner::default(),
            name: Name::new("Sheep"),
            config_set_id: ConfigurationSetId::Sheep,
            inertia: Inertia::default(),
        }
    }
}

#[derive(Default, Clone, PartialEq, Serialize, Deserialize)]
pub enum FenceOrientation {
    #[default]
    Horizontal,
    Vertical,
}

#[derive(Bundle)]
pub struct FenceBundle {
    animation_bundle: AnimationBundle,
    collider: Collider,
    rigid_body: RigidBody,
    name: Name,
    config_set_id: ConfigurationSetId,
}

impl FenceBundle {
    pub fn new(config: &animation::AnimationConfiguration, fence_orientation: &FenceOrientation, position: Vec3) -> Self {
        // let (config_set_id, dimensions)= match fence_orientation {
        //     FenceOrientation::Horizontal => { (ConfigurationSetId::FenceHorizontal, Vec2::new(5.0, 2.0) )}
        //     FenceOrientation::Vertical => { (ConfigurationSetId::FenceVertical, Vec2::new(2.0, 5.0)) }
        // };
        let config_set_id= match fence_orientation {
            FenceOrientation::Horizontal => { ConfigurationSetId::FenceHorizontal }
            FenceOrientation::Vertical => { ConfigurationSetId::FenceVertical }
        };
        let config_set = config.get_set(&config_set_id);
        let dimensions = config_set.texture_size;
            FenceBundle {
            animation_bundle: AnimationBundle::from(config_set, position),
            collider: Collider::cuboid(dimensions.x, dimensions.y),
            rigid_body: RigidBody::Fixed,
            name: Name::new("Fence"),
            config_set_id,
        }
    }
}

#[derive(Bundle)]
pub struct GrassBundle {
    animation_bundle: AnimationBundle,
    name: Name,
    config_set_id: ConfigurationSetId,
}

impl GrassBundle {
    pub fn new(config_set: &animation::AnimationSheet, position: Vec3) -> Self {
        GrassBundle {
            animation_bundle: AnimationBundle::from(config_set, position),
            name: Name::new("Grass"),
            config_set_id: ConfigurationSetId::Grass,
        }
    }
}

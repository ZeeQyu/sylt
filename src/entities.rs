pub mod sheep;
pub mod fence;
pub mod grass;
pub mod player;
pub mod sheep_cluster;
pub mod text;

use crate::imports::*;

#[derive(Bundle)]
pub struct Actor {
    pub animation_bundle: AnimationBundle,
    pub collider: Collider,
    pub rigid_body: RigidBody,
    pub locked_axes: LockedAxes,
    pub velocity: Velocity,
    pub influences: Influences,
}

impl Actor {
    fn new(config_set: &AnimationSheet, position: Vec3, collider: Collider) -> Self {
        Actor {
            animation_bundle: AnimationBundle::from(config_set, position),
            //collider: Collider::ball(15.0),
            rigid_body: RigidBody::Dynamic,
            locked_axes: LockedAxes::ROTATION_LOCKED,
            velocity: Velocity::default(),
            influences: Influences::default(),
            collider,
        }
    }
}


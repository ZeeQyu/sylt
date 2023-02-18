use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use rand_distr::{Normal, Distribution};
use crate::motion::{ConfigurationSetId, Flocking, Grazing, Influences, PlayerInput, RunsFromPlayer};

pub fn setup(
    mut commands: Commands,
    // mut meshes: ResMut<Assets<Mesh>>,
    // mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    // mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    // let texture_handle = asset_server.load("spritesheet.png");
    // let texture_atlas_handle = texture_atlases.add(texture_atlas);
    // let texture_atlas = texture_atlases.get(texture_atlas_handle).unwrap();
    // let player_sprite =
    commands.spawn(Camera2dBundle::default());
    let player_texture: Handle<Image> = asset_server.load("Collie.png");
    let sheep_texture: Handle<Image> = asset_server.load("Sheep.png");

    let normal = Normal::new(0.0, 100.0).unwrap();
    spawn_player(&mut commands, player_texture);
    for _ in 0..40 {
        let x = normal.sample(&mut rand::thread_rng());
        let y = normal.sample(&mut rand::thread_rng());
        spawn_sheep(&mut commands, Vec3::new(x, y, 0.0), sheep_texture.clone());
    }

    // for x_counter in -3..3 {
    //     for y_counter in -3..3 {
    //         let spacing: f32 = 50.0;
    //         let position = Vec3::new(
    //             (x_counter as f32) * spacing + 25.0,
    //             (y_counter as f32) * spacing + 25.0,
    //             0.0,
    //         );
    //         spawn_sheep(&mut commands, position, sheep_texture.clone());
    //     }
    // }
}

#[derive(Bundle)]
struct Common {
    sprite_bundle: SpriteBundle,
    collider: Collider,
    rigid_body: RigidBody,
    locked_axes: LockedAxes,
    velocity: Velocity,
    influences: Influences,
}

fn spawn_common(image: Handle<Image>, position: Vec3) -> Common {
    Common {
        sprite_bundle: SpriteBundle {
            transform: Transform {
                translation: Vec3::new(position.x, position.y, 0.0),
                ..default()
            },
            texture: image,
            sprite: Sprite {
                custom_size: Some(Vec2::splat(32.0)),
                ..default()
            },
            ..default()
        },
        collider: Collider::ball(15.0),
        rigid_body: RigidBody::Dynamic,
        locked_axes: LockedAxes::ROTATION_LOCKED,
        velocity: Velocity::default(),
        influences: Influences::default(),
    }
}

fn spawn_player(
    commands: &mut Commands,
    image: Handle<Image>,
) {
    commands.spawn((
        spawn_common(image, Vec3::new(-400.0, 0.0, 0.0)),
        PlayerInput {},
        Dominance::group(10),
        Name::new("Player"),
        ConfigurationSetId::Player,
    ));
}

fn spawn_sheep(
    commands: &mut Commands,
    position: Vec3,
    image: Handle<Image>,
) {
    commands.spawn(
        (
            spawn_common(image, position),
            Flocking::default(),
            Grazing {
                current_direction: None,
                time_left: (rand::random::<f32>() * 5.0) + 3.0,
            },
            RunsFromPlayer {
                direction: Vec3::ZERO,
                magnitude: 0.0,
            },
            Name::new("Sheep"),
            ConfigurationSetId::Sheep,
        )
    );
}


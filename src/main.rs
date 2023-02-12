use bevy::{
    prelude::*,
    time::FixedTimestep,
};
use sepax2d::prelude::*;
use bevy_sepax2d::prelude::*;

const TIME_STEP: f32 = 1.0 / 60.0;

fn main() {
    let background_color = Color::rgb_u8(46 as u8, 34 as u8, 47 as u8);
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugin(SepaxPlugin)
        .insert_resource(ClearColor(background_color))
        .add_startup_system(setup)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(TIME_STEP as f64))
                .with_system(apply_player_input)
                .with_system(flee_from_players)
                .with_system(calculate_velocity)
                .with_system(update_colliders)
                .with_system(perform_collisions)
                .with_system(find_flocking_neighbours)
        )
        .run();
}

fn setup(
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
    spawn_player(&mut commands, player_texture);
    for x_counter in -3..3 {
        for y_counter in -3..3 {
            let spacing: f32 = 50.0;
            let position = Vec2::new(
                (x_counter as f32) * &spacing,
                (y_counter as f32) * &spacing,
            );
            spawn_sheep(&mut commands, &position, sheep_texture.clone());
        }
    }
}

#[derive(Component)]
struct PlayerInput {
    direction: Vec3,
}

#[derive(Component)]
struct RunsFromPlayer {
    direction: Vec3,
    magnitude: f32,
}

#[derive(Component)]
struct Flocking {
    neighbour_positions: Vec<Vec3>,
    neighbour_velocities: Vec<Vec3>,
    alignment: Vec3,
    separation: Vec3,
    cohesion: Vec3,
}

#[derive(Component)]
struct YieldsToCollision {
    direction: Vec3,
}

#[derive(Component)]
struct Velocity {
    max_speed: f32,
    velocity: Vec3,
}

impl Velocity {
    fn new(max_speed: f32) -> Self {
        Self {
            max_speed,
            velocity: Vec3::ZERO,
        }
    }
}

fn spawn_player(commands: &mut Commands, player_texture: Handle<Image>) {
    let player_speed: f32 = 200.0;
    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: Vec3::new(-400.0, 0.0, 0.0),
                scale: Vec3::splat(2.0),
                ..default()
            },
            texture: player_texture,
            ..default()
        },
        PlayerInput { direction: Vec3::ZERO },
        Velocity::new(player_speed),
    ));
}

fn spawn_sheep(commands: &mut Commands, position: &Vec2, image: Handle<Image>) {
    commands.spawn(
        (
            SpriteBundle {
                transform: Transform {
                    translation: Vec3::new(position.x, position.y, 0.0),
                    scale: Vec3::splat(
                        2.0),
                    ..default()
                },
                texture: image,
                ..default()
            },
            Flocking {
                neighbour_positions: Vec::new(),
                neighbour_velocities: Vec::new(),
                alignment: Vec3::ZERO,
                separation: Vec3::ZERO,
                cohesion: Vec3::ZERO,
            },
            YieldsToCollision {
                direction: Vec3::ZERO,
            },
            RunsFromPlayer {
                direction: Vec3::ZERO,
                magnitude: 0.0,
            },
            Sepax {
                convex: Convex::Circle(
                    Circle {
                        position: (0.0, 0.0),
                        radius: 40.0,
                    }
                ),
            },
            Velocity::new(150.0),
        )
    );
}

fn apply_player_input(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut PlayerInput>,
) {
    let mut direction = Vec2::new(0.0, 0.0);
    if keyboard_input.pressed(KeyCode::Left) ||
        keyboard_input.pressed(KeyCode::A) {
        direction.x -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::Right) ||
        keyboard_input.pressed(KeyCode::D) {
        direction.x += 1.0;
    }
    if keyboard_input.pressed(KeyCode::Up) ||
        keyboard_input.pressed(KeyCode::W) {
        direction.y += 1.0;
    }
    if keyboard_input.pressed(KeyCode::Down) ||
        keyboard_input.pressed(KeyCode::S) {
        direction.y -= 1.0;
    }
    direction = direction.normalize_or_zero();
    let mut player_input = query.single_mut();
    player_input.direction = Vec3::new(
        direction.x,
        direction.y,
        0.0,
    );
}

fn flee_from_players(
    player_query: Query<(&Transform, With<PlayerInput>)>,
    mut runner_query: Query<((&mut RunsFromPlayer, &Transform), Without<PlayerInput>)>,
) {
    let (Transform { translation: player_position, .. }, ()) = player_query.single();
    for (
        (
            mut runner,
            &Transform { translation: sheep_position, .. }
        ),
        ()
    ) in runner_query.iter_mut() {
        let scare_distance = 200.0;
        if sheep_position.distance(*player_position) < scare_distance {
            runner.direction = (*player_position - sheep_position).normalize_or_zero();
            runner.magnitude = 1.0;
        } else {
            runner.magnitude -= 1.0 / TIME_STEP;
        }
    }
}
/// Uses the velocity of last frame to find neighbour velocities for flocking behaviour
fn find_flocking_neighbours(
    mut query: Query<(&Transform, &mut Flocking), With<Velocity>>,
    other_query: Query<(&Transform, &Velocity), With<Flocking>>,
) {
    for (current_transform, mut current_flocking) in query.iter_mut() {
        let current_transform: &Transform = current_transform;
        current_flocking.neighbour_positions.clear();
        current_flocking.neighbour_velocities.clear();
        for (other_transform, other_velocity) in other_query.iter() {
            let neighbour_distance = 400.0;
            if current_transform.translation.distance(other_transform.translation) < neighbour_distance {
                current_flocking.neighbour_positions.push(other_transform.translation);
                current_flocking.neighbour_velocities.push(other_velocity.velocity);
            }
        }
    }
}

fn calculate_flocking(
    mut query: Query<(&mut Flocking), With<Velocity>>,
) {
    for (mut flocking) in query.iter_mut() {
        let mut flocking: &mut Flocking = &mut flocking;
        flocking.alignment = Vec3::ZERO;
        flocking.cohesion = Vec3::ZERO;
        flocking.separation = Vec3::ZERO;
        for velocity in flocking.neighbour_velocities.iter() {
            flocking.alignment += velocity.normalize_or_zero();
        }
        for position in flocking.neighbour_positions.iter() {
            // flocking.cohesion +=
        }
    }
}

fn calculate_velocity(
    mut query: Query<(
        &mut Transform,
        &mut Velocity,
        Option<&RunsFromPlayer>,
        Option<&PlayerInput>,
        Option<&Flocking>
    )>
) {
    for (mut transform, mut velocity, runner, player, flocker) in query.iter_mut() {
        let mut influence = Vec3::ZERO;
        if let Some(player) = player {
            influence = player.direction;
        }
        if let Some(runner) = runner {
            influence = runner.direction;
        }
        if let Some(flocker) = flocker {

        }
        influence = influence.normalize_or_zero();
        velocity.velocity = influence * velocity.max_speed;
        transform.translation += velocity.velocity * TIME_STEP;
    }
}

pub fn update_colliders(mut query: Query<(&Transform, &mut Sepax)>)
{
    for (transform, mut sepax) in query.iter_mut()
    {
        let position = (transform.translation.x, transform.translation.y);

        let shape = sepax.shape_mut();
        shape.set_position(position);
    }
}

pub fn perform_collisions(mut _query: Query<(&mut Sepax, &mut Transform), Without<NoCollision>>)
{
    // for (mut sepax, mut transform) in movable.iter_mut()
    // {
    //     for wall in walls.iter()
    //     {
    //         let shape = sepax.shape_mut();
    //         let correction = sat_collision(wall.shape(), shape);
    //
    //         let old_position = shape.position();
    //         let new_position = (old_position.0 + correction.0, old_position.1 + correction.1);
    //
    //         shape.set_position(new_position);
    //         transform.translation.x = new_position.0;
    //         transform.translation.y = new_position.1;
    //
    //         let length = f32::sqrt((correction.0 * correction.0) + (correction.1 * correction.1));
    //
    //         if length > f32::EPSILON
    //         {
    //             // correct.axes.push((correction.0 / length, correction.1 / length));
    //         }
    //     }
    // }
}

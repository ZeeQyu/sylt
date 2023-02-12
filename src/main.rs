use std::cmp::max;
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
                .with_system(apply_player_input.before(calculate_velocity))
                .with_system(flee_from_players.before(calculate_velocity))
                .with_system(update_colliders.before(calculate_collisions))
                .with_system(calculate_collisions.before(calculate_velocity))
                // .with_system(apply_collision_corrections.before(calculate_velocity))
                .with_system(calculate_velocity)
            // .with_system(find_flocking_neighbours)
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
                (x_counter as f32) * spacing,
                (y_counter as f32) * spacing,
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
    correction: Vec3,
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
                correction: Vec3::ZERO,
            },
            RunsFromPlayer {
                direction: Vec3::ZERO,
                magnitude: 0.0,
            },
            Sepax {
                convex: Convex::Circle(
                    Circle {
                        position: (position.x, position.y),
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
    player_query: Query<&Transform, With<PlayerInput>>,
    mut runner_query: Query<(&mut RunsFromPlayer, &Transform), Without<PlayerInput>>,
) {
    let Transform { translation: player_position, .. } = player_query.single();
    for (mut runner, Transform { translation: runner_position, .. }) in runner_query.iter_mut() {
        let mut runner: &mut RunsFromPlayer = &mut runner;

        let scare_distance = 200.0;
        if runner_position.distance(*player_position) < scare_distance {
            // runner.direction = (*runner_position - *player_position).normalize_or_zero();
            runner.direction = (*player_position - *runner_position).normalize_or_zero();
            runner.magnitude = 1.0;
        } else {
            runner.magnitude -= 1.0 * TIME_STEP;
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
        Option<&Flocking>,
        Option<&YieldsToCollision>,
    )>
) {
    for (mut transform, mut velocity, runner, player, flocker, yields) in query.iter_mut() {
        let mut transform: &mut Transform = &mut transform;
        let mut velocity: &mut Velocity = &mut velocity;
        let runner: Option<&RunsFromPlayer> = runner;
        let player: Option<&PlayerInput> = player;
        let flocker: Option<&Flocking> = flocker;
        let yields: Option<&YieldsToCollision> = yields;
        let mut influence = Vec3::ZERO;
        if let Some(player) = player {
            influence = player.direction;
        }
        if let Some(runner) = runner {
            let direction = runner.direction.normalize_or_zero();
            influence = direction * f32::max(runner.magnitude, 0.0);
        }
        if let Some(flocker) = flocker {}
        let influence_length = influence.length();
        if influence_length > 1.0 {
            influence /= influence_length;
        }
        if let Some(yields) = yields {
            let correction_direction = yields.correction;//.normalize_or_zero();
            influence += correction_direction * 0.9;
        }
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

fn calculate_collisions(
    mut query: Query<(Entity, &Sepax, &mut YieldsToCollision), Without<NoCollision>>,
    movables: Query<(Entity, &Sepax), (With<YieldsToCollision>, Without<NoCollision>)>,
    immovables: Query<&Sepax, (Without<YieldsToCollision>, Without<NoCollision>)>,
)
{
    let mut items: Vec<Entity> = Vec::new();
    let mut others: Vec<Entity> = Vec::new();
    for (entity, sepax, mut yields) in query.iter_mut() {
        items.push(entity);
        let sepax: &Sepax = sepax;
        let mut yields: &mut YieldsToCollision = &mut yields;
        others.clear();
        yields.correction = Vec3::ZERO;
        for (other_entity, other_sepax) in movables.iter() {
            others.push(other_entity);
            let other_sepax: &Sepax = other_sepax;
            if entity != other_entity {
                let correction = sat_collision(other_sepax.shape(), sepax.shape());
                // We divide collisions with other things that yield by 2.
                // This is since the other side will also adjust to us, meaning we'd otherwise
                // overcorrect
                yields.correction += Vec3::new(correction.0, correction.1, 0.0) / 2.0;
            }
        }
        for other_sepax in immovables.iter() {
            let correction = sat_collision(other_sepax.shape(), sepax.shape());
            yields.correction += Vec3::new(correction.0, correction.1, 0.0);
        }
    }
    // println!("items: {}", items.len());
    // println!("others: {}", others.len());
    // println!("{:#?}", items);
    // println!("{:#?}", others);
}

fn apply_collision_corrections(
    mut query: Query<(&mut Transform, &YieldsToCollision), Without<NoCollision>>,
) {
    for (mut transform, yields) in query.iter_mut() {
        let mut transform: &mut Transform = &mut transform;
        let yields: &YieldsToCollision = &yields;
        transform.translation += yields.correction;
    }
}

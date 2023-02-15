use bevy::{
    prelude::*,
    time::FixedTimestep,
};
use bevy_rapier2d::prelude::*;
use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::quick::{ResourceInspectorPlugin, WorldInspectorPlugin};

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
    alignment_direction: Option<Vec3>,
    separation_vector: Option<Vec3>,
    cohesion_correction: Option<Vec3>,
}

#[derive(Component)]
enum ConfigurationSetId {
    Player,
    Sheep,
}

#[derive(Reflect, Default, InspectorOptions)]
#[reflect(InspectorOptions)]
struct ConfigurationSet {
    #[inspector(min = 0.0)]
    max_speed: f32,
}

#[derive(Reflect, Default, InspectorOptions)]
#[reflect(InspectorOptions)]
struct FlockingConfiguration {
    neighbour_distance: f32,
    #[inspector(min = 0.0)]
    alignment_weight: f32,
    #[inspector(min = 0.0)]
    cohesion_weight: f32,
    #[inspector(min = 0.0)]
    separation_weight: f32,
}

#[derive(Reflect, Default, InspectorOptions)]
#[reflect(InspectorOptions)]
struct RunnerConfiguration {
    #[inspector(min = 0.0)]
    weight: f32,
    #[inspector(min = 0.0)]
    added_speed: f32,
    #[inspector(min = 0.0)]
    scare_distance: f32,
}

#[derive(Reflect, Default, Resource, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
struct Configuration {
    player: ConfigurationSet,
    sheep: ConfigurationSet,
    flocking: FlockingConfiguration,
    runner: RunnerConfiguration,
}
impl Configuration {
    fn new() -> Self {
        Self {
            player: ConfigurationSet {
                max_speed: 200.0,
            },
            sheep: ConfigurationSet {
                max_speed: 150.0,
            },
            flocking: FlockingConfiguration {
                neighbour_distance: 200.0,
                alignment_weight: 1.0,
                cohesion_weight: 1.0,
                separation_weight: 1.0,
            },
            runner: RunnerConfiguration {
                weight: 0.0,
                added_speed: 100.0,
                scare_distance: 100.0,
            },
        }
    }
    fn get_set<'a>(self: &'a Self, id: &ConfigurationSetId) -> &'a ConfigurationSet {
        match id {
            ConfigurationSetId::Player => {
                &self.player
            }
            ConfigurationSetId::Sheep => {
                &self.sheep
            }
        }
    }
}

const TIME_STEP: f32 = 1.0 / 60.0;

fn main() {
    let background_color = Color::rgb_u8(46 as u8, 34 as u8, 47 as u8);
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(30.0))
        // .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(WorldInspectorPlugin)

        .register_type::<Configuration>()
        .insert_resource::<Configuration>(Configuration::new())
        .add_plugin(ResourceInspectorPlugin::<Configuration>::default())

        .insert_resource(ClearColor(background_color))
        .insert_resource(RapierConfiguration { gravity: Vec2::ZERO, ..default() })
        .add_startup_system(setup)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(TIME_STEP as f64))
                .with_system(apply_player_input.before(calculate_velocity))
                .with_system(flee_from_player.before(calculate_velocity))
                .with_system(find_flocking_neighbours.before(calculate_flocking))
                .with_system(calculate_flocking.before(calculate_velocity))
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
                (x_counter as f32) * spacing + 25.0,
                (y_counter as f32) * spacing + 25.0,
            );
            spawn_sheep(&mut commands, &position, sheep_texture.clone());
        }
    }
}

fn spawn_player(
    commands: &mut Commands,
    player_texture: Handle<Image>,
) {
    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: Vec3::new(-400.0, 0.0, 0.0),
                ..default()
            },
            texture: player_texture,
            sprite: Sprite {
                custom_size: Some(Vec2::splat(32.0)),
                ..default()
            },
            ..default()
        },
        PlayerInput { direction: Vec3::ZERO },
        Collider::ball(15.0),
        Dominance::group(10),
        RigidBody::Dynamic,
        LockedAxes::ROTATION_LOCKED,
        Velocity::default(),
        Name::new("Player"),
        ConfigurationSetId::Player,
    ));
}

fn spawn_sheep(
    commands: &mut Commands,
    position: &Vec2,
    image: Handle<Image>,
) {
    // commands
    //     .spawn(Collider::cuboid(500.0, 50.0))
    //     .insert(TransformBundle::from(
    //         Transform::from_xyz(0.0, -100.0, 0.0)
    //     ));
    // commands
    //     .spawn(RigidBody::Dynamic)
    //     .insert(Collider::ball(50.0))
    //     .insert(Restitution::coefficient(0.7))
    //     .insert(TransformBundle::from(Transform::from_xyz(0.0, 400.0, 0.0)));
    commands.spawn(
        (
            SpriteBundle {
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
            Collider::ball(13.0),
            RigidBody::Dynamic,
            LockedAxes::ROTATION_LOCKED,
            Velocity::default(),
            Flocking {
                neighbour_positions: Vec::new(),
                neighbour_velocities: Vec::new(),
                alignment_direction: None,
                cohesion_correction: None,
                separation_vector: None,
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

fn flee_from_player(
    player_query: Query<&Transform, With<PlayerInput>>,
    mut runner_query: Query<(&mut RunsFromPlayer, &Transform), Without<PlayerInput>>,
    config: Res<Configuration>,
) {
    let Transform { translation: player_position, .. } = player_query.single();
    for (mut runner, Transform { translation: runner_position, .. }) in runner_query.iter_mut() {
        let mut runner: &mut RunsFromPlayer = &mut runner;

        let scare_distance = config.runner.scare_distance;
        if runner_position.distance(*player_position) < scare_distance {
            runner.direction = (*runner_position - *player_position).normalize_or_zero();
            // runner.direction = (*player_position - *runner_position).normalize_or_zero();
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
    config: Res<Configuration>,
) {
    for (current_transform, mut current_flocking) in query.iter_mut() {
        let current_transform: &Transform = current_transform;
        current_flocking.neighbour_positions.clear();
        current_flocking.neighbour_velocities.clear();
        for (other_transform, other_velocity) in other_query.iter() {
            if current_transform.translation.distance(other_transform.translation) < config.flocking.neighbour_distance {
                current_flocking.neighbour_positions.push(other_transform.translation);
                current_flocking.neighbour_velocities.push(Vec3::from((other_velocity.linvel, 0.0)));
            }
        }
    }
}

fn calculate_flocking(
    mut query: Query<(&mut Flocking, &Transform), With<Velocity>>,
) {
    for (mut flocking, transform) in query.iter_mut() {
        if flocking.neighbour_velocities.len() > 0 {
            // Alignment
            let mut alignment = Vec3::ZERO;
            for velocity in flocking.neighbour_velocities.iter() {
                alignment += velocity.normalize_or_zero();
            }
            alignment /= flocking.neighbour_velocities.len() as f32;
            flocking.alignment_direction = Some(alignment);

            // Cohesion
            let mut cohesion_center = Vec3::ZERO;
            for position in flocking.neighbour_positions.iter() {
                cohesion_center += *position;
            }
            cohesion_center /= flocking.neighbour_positions.len() as f32;
            let correction_to_center = cohesion_center - transform.translation;
            flocking.cohesion_correction = Some(correction_to_center);

            // Separation
            let mut separation = Vec3::ZERO;
            for position in flocking.neighbour_positions.iter() {
                let to_neighbour: Vec3 = *position - transform.translation;
                let distance_recip = to_neighbour.length_recip();
                separation += -to_neighbour / distance_recip;
            }
            flocking.separation_vector = Some(separation);
        } else {
            flocking.alignment_direction = None;
            flocking.cohesion_correction = None;
            flocking.separation_vector = None;
        }
    }
}

fn calculate_velocity(
    mut query: Query<(
        &mut Velocity,
        &ConfigurationSetId,
        Option<&RunsFromPlayer>,
        Option<&PlayerInput>,
        Option<&Flocking>,
    )>,
    config: Res<Configuration>,
) {
    for (
        mut velocity,
        set_id,
        runner,
        player,
        flocker
    ) in query.iter_mut() {
        let set = config.get_set(set_id);
        let mut influence = Vec3::ZERO;
        if let Some(player) = player {
            influence += player.direction;
        }
        if let Some(flocker) = flocker {
            let config = &config.flocking;
            if let Some(alignment_direction) = flocker.alignment_direction {
                influence += alignment_direction * config.alignment_weight;
            }
            if let Some(correction) = flocker.cohesion_correction {
                influence += correction * config.cohesion_weight;
            }
            if let Some(separation_vector) = flocker.separation_vector {
                influence += separation_vector * config.separation_weight / 10000.0;
            }
        }
        let influence_length = influence.length();
        if influence_length > 1.0 {
            influence /= influence_length;
        }
        let mut runner_influence = Vec3::ZERO;
        if let Some(runner) = runner {
            let direction = runner.direction.normalize_or_zero();
            runner_influence = direction * f32::max(runner.magnitude, 0.0);// * config.runner_weight;
        }
        let influence: Vec2 = Vec2::new(influence.x, influence.y);
        let runner_influnce: Vec2 = Vec2::new(runner_influence.x, runner_influence.y);
        velocity.linvel = influence * set.max_speed + runner_influnce * config.runner.added_speed;
    }
}
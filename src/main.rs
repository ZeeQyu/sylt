use bevy::{
    prelude::*,
    time::FixedTimestep,
};
use bevy_rapier2d::prelude::*;
use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::quick::{ResourceInspectorPlugin, WorldInspectorPlugin};
use bevy_prototype_debug_lines::*;

#[derive(Component)]
struct PlayerInput {}

#[derive(Component)]
struct RunsFromPlayer {
    direction: Vec3,
    magnitude: f32,
}

struct NeighbourPair {
    position: Vec3,
    velocity: Vec3,
}

#[derive(Component, Default)]
struct Flocking {
    alignment_values: Vec<NeighbourPair>,
    cohesion_positions: Vec<Vec3>,
    separation_positions: Vec<Vec3>,
}

#[derive(Component, Default)]
struct Grazing {
    current_direction: Option<Vec3>,
    time_left: f32,
}

#[derive(Component, Default)]
struct Influences {
    player_input_influence: Option<Vec3>,
    alignment_influence: Option<Vec3>,
    cohesion_influence: Option<Vec3>,
    separation_influence: Option<Vec3>,
    runner_influence: Option<Vec3>,
    runner_influence_unmodified: Option<Vec3>,
    runner_influence_max: Option<Vec3>,
    grazing_influence: Option<Vec3>,
    total_influence: Option<Vec3>,
    max_influence: Option<Vec3>,
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
    #[inspector(min = 0.0)]
    alignment_distance: f32,
    #[inspector(min = 0.0)]
    alignment_scale: f32,
    #[inspector(min = 0.0)]
    alignment_distance_cap_fraction: f32,
    #[inspector(min = 0.0)]
    cohesion_distance: f32,
    #[inspector(min = 0.0)]
    cohesion_scale: f32,
    #[inspector(min = 0.0)]
    separation_distance: f32,
    #[inspector(min = 0.0)]
    separation_scale: f32,
}

#[derive(Reflect, Default, InspectorOptions)]
#[reflect(InspectorOptions)]
struct RunnerConfiguration {
    #[inspector(min = 0.0)]
    scale: f32,
    #[inspector(min = 0.0)]
    speed_fraction: f32,
    #[inspector(min = 0.0)]
    scare_distance: f32,
}

#[derive(Reflect, Default)]
enum DebugLineType {
    #[default]
    None,
    AlignmentInfluence,
    CohesionInfluence,
    SeparationInfluence,
    RunnerInfluence,
    RunnerUnmodifiedInfluence,
    RunnerMaxInfluence,
    TotalInfluence,
    MaxInfluence,
}

#[derive(Reflect, Default)]
struct DebugLineConfiguration {
    enable: bool,
    red: DebugLineType,
    green: DebugLineType,
    blue: DebugLineType,
    gray: DebugLineType,
}

#[derive(Reflect, Default, Resource, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
struct Configuration {
    player: ConfigurationSet,
    sheep: ConfigurationSet,
    flocking: FlockingConfiguration,
    runner: RunnerConfiguration,
    grazing_scale: f32,
    debug_lines: DebugLineConfiguration,
}

impl Configuration {
    fn new() -> Self {
        Self {
            player: ConfigurationSet {
                max_speed: 300.0,
            },
            sheep: ConfigurationSet {
                max_speed: 100.0,
            },
            flocking: FlockingConfiguration {
                alignment_distance: 50.0,
                alignment_scale: 80.0,
                alignment_distance_cap_fraction: 0.4,
                cohesion_distance: 200.0,
                cohesion_scale: 0.0,
                separation_distance: 30.0,
                separation_scale: 30.0,
            },
            runner: RunnerConfiguration {
                scale: 10.0,
                speed_fraction: 1.4,
                scare_distance: 100.0,
            },
            grazing_scale: 1.0,
            debug_lines: DebugLineConfiguration {
                enable: false,
                // red: DebugLineType::None,
                // green: DebugLineType::None,
                // blue: DebugLineType::None,
                // gray: DebugLineType::None,
                red: DebugLineType::AlignmentInfluence,
                green: DebugLineType::CohesionInfluence,
                blue: DebugLineType::SeparationInfluence,
                gray: DebugLineType::TotalInfluence,
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
        .add_plugin(DebugLinesPlugin::default())

        .register_type::<Configuration>()
        .insert_resource::<Configuration>(Configuration::new())
        .add_plugin(ResourceInspectorPlugin::<Configuration>::default())

        .insert_resource(ClearColor(background_color))
        .insert_resource(RapierConfiguration { gravity: Vec2::ZERO, ..default() })
        .add_startup_system(setup)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(TIME_STEP as f64))
                .with_system(reset_influences)
                .with_system(apply_player_input.before(calculate_velocity).after(reset_influences))
                .with_system(run_from_players.before(calculate_velocity).after(reset_influences))
                .with_system(find_flocking_neighbours.before(calculate_flocking))
                .with_system(calculate_flocking.before(calculate_velocity).after(reset_influences))
                .with_system(calculate_grazing.before(calculate_velocity).after(reset_influences))
                .with_system(calculate_velocity)
                .with_system(draw_debug_lines.after(calculate_velocity))
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
        PlayerInput {},
        Collider::ball(15.0),
        Dominance::group(10),
        RigidBody::Dynamic,
        LockedAxes::ROTATION_LOCKED,
        Velocity::default(),
        Influences::default(),
        Name::new("Player"),
        ConfigurationSetId::Player,
    ));
}

fn spawn_sheep(
    commands: &mut Commands,
    position: &Vec2,
    image: Handle<Image>,
) {
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
            Influences::default(),
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

fn reset_influences(mut query: Query<&mut Influences>) {
    for mut influence in query.iter_mut() {
        *influence = Influences::default();
    }
}

fn apply_player_input(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Influences, &PlayerInput)>,
) {
    let mut direction = Vec3::new(0.0, 0.0, 0.0);
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
    let (mut influences, _) = query.single_mut();
    influences.player_input_influence = Some(direction);
}

fn run_from_players(
    player_query: Query<&Transform, With<PlayerInput>>,
    mut runner_query: Query<(&mut RunsFromPlayer, &mut Influences, &Transform), Without<PlayerInput>>,
    config: Res<Configuration>,
) {
    let Transform { translation: player_position, .. } = player_query.single();
    for (
        mut runner,
        mut influences,
        Transform { translation: runner_position, .. }
    ) in runner_query.iter_mut() {
        let mut runner: &mut RunsFromPlayer = &mut runner;

        let scare_distance = config.runner.scare_distance;
        if runner_position.distance(*player_position) < scare_distance {
            runner.direction = (*runner_position - *player_position).normalize_or_zero();
            // runner.direction = (*player_position - *runner_position).normalize_or_zero();
            if runner.magnitude < 0.5 {
                runner.magnitude = 0.5
            } else {
                runner.magnitude = f32::min(runner.magnitude + 1.0 * TIME_STEP, 1.0);
            }
        } else {
            runner.magnitude -= 1.0 * TIME_STEP;
        }
        if runner.magnitude >= f32::EPSILON {
            let mut influence = runner.direction * runner.magnitude * config.runner.scale / 10.0;
            influences.runner_influence_unmodified = Some(influence);
            let influence_length = influence.length();
            if influence_length > config.runner.speed_fraction {
                influence *= config.runner.speed_fraction / influence_length;
            }
            let influence_max = influence * config.runner.speed_fraction / influence_length;
            influences.runner_influence_max = Some(influence_max);
            influences.runner_influence = Some(influence);
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
        current_flocking.alignment_values.clear();
        current_flocking.cohesion_positions.clear();
        current_flocking.separation_positions.clear();
        for (other_transform, other_velocity) in other_query.iter() {
            let distance = current_transform.translation.distance(other_transform.translation);
            if distance < config.flocking.alignment_distance {
                current_flocking.alignment_values.push(NeighbourPair {
                    position: other_transform.translation,
                    velocity: Vec3::from((other_velocity.linvel, 0.0)),
                });
            }
            if distance < config.flocking.cohesion_distance {
                current_flocking.cohesion_positions.push(other_transform.translation);
            }
            if distance < config.flocking.separation_distance {
                current_flocking.separation_positions.push(other_transform.translation);
            }
        }
    }
}

fn calculate_flocking(
    mut query: Query<(&mut Influences, &Flocking, &Transform), With<Velocity>>,
    configuration: Res<Configuration>,
) {
    for (mut influences, flocking, transform) in query.iter_mut() {
        // Alignment
        if flocking.alignment_values.len() > 0 {
            let mut alignment = Vec3::ZERO;
            for NeighbourPair { position, velocity } in flocking.alignment_values.iter() {
                let distance = position.distance(transform.translation);
                let max_distance = configuration.flocking.alignment_distance;
                let distance_scale = (max_distance - distance) / max_distance;
                let distance_scale = f32::max(
                    distance_scale,
                    configuration.flocking.alignment_distance_cap_fraction,
                );
                alignment += *velocity * distance_scale;
            }
            alignment /= flocking.alignment_values.len() as f32;
            influences.alignment_influence = Some(
                alignment * configuration.flocking.alignment_scale / 10000.0
            );
        }

        // Cohesion
        if flocking.cohesion_positions.len() > 0 {
            let mut cohesion_center = Vec3::ZERO;
            for position in flocking.cohesion_positions.iter() {
                cohesion_center += *position;
            }
            cohesion_center /= flocking.cohesion_positions.len() as f32;
            let correction_to_center = cohesion_center - transform.translation;
            influences.cohesion_influence = Some(
                correction_to_center * configuration.flocking.cohesion_scale / 1000.0
            );
        }

        // Separation
        if flocking.separation_positions.len() > 0 {
            let mut separation = Vec3::ZERO;
            for position in flocking.separation_positions.iter() {
                let to_neighbour: Vec3 = *position - transform.translation;
                let distance_recip = to_neighbour.length_recip();
                separation += -to_neighbour / distance_recip;
            }
            separation /= flocking.separation_positions.len() as f32;
            influences.separation_influence = Some(
                separation * configuration.flocking.separation_scale / 100000.0
            );
        }
    }
}

fn calculate_grazing(
    mut query: Query<(&mut Influences, &mut Grazing)>,
    config: Res<Configuration>,
) {
    for (mut influences, mut grazing) in query.iter_mut() {
        grazing.time_left -= TIME_STEP;
        if grazing.time_left <= 0.0 {
            if rand::random::<f32>() < 0.4 {
                let direction = Vec3::new(
                    rand::random::<f32>() - 0.5,
                    rand::random::<f32>() - 0.5,
                    rand::random::<f32>() - 0.5,
                );
                grazing.current_direction = Some(direction);
                grazing.time_left = rand::random::<f32>() * 5.0 + 0.5;
            } else {
                grazing.current_direction = None;
            }
        }
        if let Some(current_direction) = grazing.current_direction {
            influences.grazing_influence = Some(current_direction * config.grazing_scale / 10.0);
        } else {
            influences.grazing_influence = None;
        }
    }
}

fn calculate_velocity(
    mut query: Query<(
        &mut Velocity,
        &mut Influences,
        &ConfigurationSetId,
    )>,
    config: Res<Configuration>,
) {
    for (
        mut velocity,
        mut influences,
        set_id,
    ) in query.iter_mut() {
        let mut total_influence = Vec3::ZERO;

        if let Some(influence) = influences.player_input_influence {
            total_influence += influence;
        }
        if let Some(influence) = influences.alignment_influence {
            total_influence += influence;
        }
        if let Some(influence) = influences.cohesion_influence {
            total_influence += influence;
        }
        if let Some(influence) = influences.separation_influence {
            total_influence += influence;
        }
        if let Some(influence) = influences.grazing_influence {
            total_influence += influence;
        }

        let influence_length = total_influence.length();
        if influence_length > 1.0 {
            total_influence /= influence_length;
        }
        if let Some(influence) = influences.runner_influence {
            total_influence += influence;
            let influence_length = total_influence.length();
            if influence_length > config.runner.speed_fraction {
                total_influence *= config.runner.speed_fraction / influence_length;
            }
        }
        influences.total_influence = Some(total_influence);
        influences.max_influence = Some(total_influence.normalize_or_zero());
        let total_influence: Vec2 = Vec2::new(total_influence.x, total_influence.y);
        let set = config.get_set(set_id);
        velocity.linvel = total_influence * set.max_speed;
    }
}

fn draw_debug_lines(
    query: Query<(&Transform, &Influences)>,
    mut lines: ResMut<DebugLines>,
    configuration: Res<Configuration>,
) {
    let debug_lines = &configuration.debug_lines;
    let mut color_to_type = Vec::new();
    color_to_type.push((Color::RED, &debug_lines.red, -2.0));
    color_to_type.push((Color::GREEN, &debug_lines.green, 0.0));
    color_to_type.push((Color::BLUE, &debug_lines.blue, 2.0));
    color_to_type.push((Color::GRAY, &debug_lines.gray, 4.0));
    for (color, debug_line, offset_degree) in color_to_type {
        for (transform, influences) in query.iter() {
            let influence = match debug_line {
                DebugLineType::None => { None }
                DebugLineType::AlignmentInfluence => { influences.alignment_influence }
                DebugLineType::CohesionInfluence => { influences.cohesion_influence }
                DebugLineType::SeparationInfluence => { influences.separation_influence }
                DebugLineType::RunnerInfluence => { influences.runner_influence }
                DebugLineType::RunnerUnmodifiedInfluence => { influences.runner_influence_unmodified }
                DebugLineType::RunnerMaxInfluence => { influences.runner_influence_max }
                DebugLineType::TotalInfluence => { influences.total_influence }
                DebugLineType::MaxInfluence => { influences.max_influence }
            };
            if let Some(influence) = influence {
                let line_graphics_scale = 50.0;
                let offset = influence.cross(Vec3::Z) * offset_degree;
                lines.line_colored(
                    transform.translation + Vec3::Z,
                    transform.translation + influence * line_graphics_scale + Vec3::Z + offset,
                    TIME_STEP,
                    color,
                );
            }
        }
    }
}
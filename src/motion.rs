use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use crate::{TIME_STEP, ConfigurationSetId};
use bevy_prototype_debug_lines::*;
use bevy_inspector_egui::prelude::*;

// #[derive(Default)]
// pub struct MotionPlugin;
//
// impl Plugin for MotionPlugin {
//     fn build(&self, app: &mut App) {
//     }
// }

impl Configuration {
    pub fn new() -> Self {
        Self {
            zoom: 1.0,
            animation: crate::animation::AnimationConfiguration::new(),
            player: ConfigurationSet {
                max_speed: 300.0,
            },
            sheep: ConfigurationSet {
                max_speed: 100.0,
            },
            flocking: FlockingConfiguration {
                alignment_enabled: true,
                alignment_distance: 60.0,
                alignment_scale: 100.0,
                alignment_distance_cap_fraction: 0.6,
                cohesion_enabled: true,
                cohesion_velocity_scale: true,
                cohesion_distance: 300.0,
                cohesion_scale: 1.5,
                separation_enabled: true,
                separation_distance: 30.0,
                separation_scale: 30.0,
            },
            runner: RunnerConfiguration {
                scale: 10.0,
                speed_fraction: 1.4,
                scare_distance: 160.0,
            },
            grazing_scale: 1.0,
            inertia_scale: 10.0,
            debug_lines: DebugLineConfiguration {
                enable: false,
                // red: DebugLineType::None,
                // green: DebugLineType::None,
                // blue: DebugLineType::None,
                // gray: DebugLineType::None,
                red: DebugLineType::AlignmentInfluence,
                green: DebugLineType::CohesionInfluence,
                blue: DebugLineType::InertiaInfluence,
                gray: DebugLineType::TotalInfluence,
            },
        }
    }
    pub fn get_set<'a>(self: &'a Self, id: &ConfigurationSetId) -> &'a ConfigurationSet {
        match id {
            ConfigurationSetId::Player => {
                &self.player
            }
            ConfigurationSetId::Sheep => {
                &self.sheep
            }
            ConfigurationSetId::Grass => {
                &self.sheep
            }
        }
    }
}

#[derive(Reflect, Default, Resource, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
pub struct Configuration {
    pub zoom: f32,
    pub animation: crate::animation::AnimationConfiguration,
    player: ConfigurationSet,
    sheep: ConfigurationSet,
    flocking: FlockingConfiguration,
    runner: RunnerConfiguration,
    grazing_scale: f32,
    inertia_scale: f32,
    debug_lines: DebugLineConfiguration,
}

#[derive(Reflect, Default, Resource, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
pub struct FlockingConfiguration {
    alignment_enabled: bool,
    #[inspector(min = 0.0)]
    alignment_distance: f32,
    #[inspector(min = 0.0)]
    alignment_scale: f32,
    #[inspector(min = 0.0)]
    alignment_distance_cap_fraction: f32,
    cohesion_enabled: bool,
    cohesion_velocity_scale: bool,
    #[inspector(min = 0.0)]
    cohesion_distance: f32,
    #[inspector(min = 0.0)]
    cohesion_scale: f32,
    separation_enabled: bool,
    #[inspector(min = 0.0)]
    separation_distance: f32,
    #[inspector(min = 0.0)]
    separation_scale: f32,
}

#[derive(Reflect, Default, InspectorOptions)]
#[reflect(InspectorOptions)]
pub struct RunnerConfiguration {
    #[inspector(min = 0.0)]
    scale: f32,
    #[inspector(min = 0.0)]
    speed_fraction: f32,
    #[inspector(min = 0.0)]
    scare_distance: f32,
}

#[derive(Component)]
pub struct PlayerInput {}

#[derive(Component)]
pub struct RunsFromPlayer {
    pub direction: Vec3,
    pub magnitude: f32,
}

pub struct NeighbourPair {
    position: Vec3,
    velocity: Vec3,
}

#[derive(Component, Default)]
pub struct Flocking {
    pub alignment_values: Vec<NeighbourPair>,
    pub cohesion_positions: Vec<Vec3>,
    pub separation_positions: Vec<Vec3>,
}

#[derive(Component, Default)]
pub struct Grazing {
    pub current_direction: Option<Vec3>,
    pub time_left: f32,
}

#[derive(Component, Default)]
pub struct Inertia();

#[derive(Component, Default)]
pub struct Influences {
    pub player_input_influence: Option<Vec3>,
    pub alignment_influence: Option<Vec3>,
    pub cohesion_influence: Option<Vec3>,
    pub separation_influence: Option<Vec3>,
    pub runner_influence: Option<Vec3>,
    pub runner_influence_unmodified: Option<Vec3>,
    pub runner_influence_max: Option<Vec3>,
    pub grazing_influence: Option<Vec3>,
    pub inertia_influence: Option<Vec3>,
    pub total_influence: Option<Vec3>,
    pub max_influence: Option<Vec3>,
}

#[derive(Reflect, Default, InspectorOptions)]
#[reflect(InspectorOptions)]
pub struct ConfigurationSet {
    #[inspector(min = 0.0)]
    pub max_speed: f32,
}

#[derive(Reflect, Default)]
pub enum DebugLineType {
    #[default]
    None,
    AlignmentInfluence,
    CohesionInfluence,
    SeparationInfluence,
    RunnerInfluence,
    RunnerUnmodifiedInfluence,
    RunnerMaxInfluence,
    GrazingInfluence,
    InertiaInfluence,
    TotalInfluence,
    MaxInfluence,
}

#[derive(Reflect, Default)]
pub struct DebugLineConfiguration {
    enable: bool,
    red: DebugLineType,
    green: DebugLineType,
    blue: DebugLineType,
    gray: DebugLineType,
}

pub fn reset_influences(mut query: Query<&mut Influences>) {
    for mut influence in query.iter_mut() {
        *influence = Influences::default();
    }
}

pub fn apply_player_input(
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

pub fn run_from_players(
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
pub fn find_flocking_neighbours(
    mut query: Query<(Entity, &Transform, &mut Flocking), With<Velocity>>,
    other_query: Query<(Entity, &Transform, &Velocity), With<Flocking>>,
    config: Res<Configuration>,
) {
    let config = &config.flocking;
    if config.alignment_enabled || config.cohesion_enabled || config.separation_enabled {
        for (entity, current_transform, mut current_flocking) in query.iter_mut() {
            let current_transform: &Transform = current_transform;
            current_flocking.alignment_values.clear();
            current_flocking.cohesion_positions.clear();
            current_flocking.separation_positions.clear();
            for (other_entity, other_transform, other_velocity) in other_query.iter() {
                if entity == other_entity { continue; }
                let distance = current_transform.translation.distance(other_transform.translation);
                if distance < config.alignment_distance && config.alignment_enabled {
                    current_flocking.alignment_values.push(NeighbourPair {
                        position: other_transform.translation,
                        velocity: Vec3::from((other_velocity.linvel, 0.0)),
                    });
                }
                if distance < config.cohesion_distance && config.cohesion_enabled {
                    current_flocking.cohesion_positions.push(other_transform.translation);
                }
                if distance < config.separation_distance && config.separation_enabled {
                    current_flocking.separation_positions.push(other_transform.translation);
                }
            }
        }
    }
}

pub fn calculate_flocking(
    mut query: Query<(&mut Influences, &Flocking, &Transform, &Velocity)>,
    config: Res<Configuration>,
) {
    for (
        mut influences,
        flocking,
        transform,
        velocity,
    ) in query.iter_mut() {
        // Alignment
        if flocking.alignment_values.len() > 0 {
            let mut alignment = Vec3::ZERO;
            for NeighbourPair { position, velocity } in flocking.alignment_values.iter() {
                let distance = position.distance(transform.translation);
                let max_distance = config.flocking.alignment_distance;
                let distance_scale = (max_distance - distance) / max_distance;
                let distance_scale = f32::max(
                    distance_scale,
                    config.flocking.alignment_distance_cap_fraction,
                );
                alignment += *velocity * distance_scale;
            }
            alignment /= flocking.alignment_values.len() as f32;
            influences.alignment_influence = Some(
                alignment * config.flocking.alignment_scale / 10000.0
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
            let velocity_scale;
            if config.flocking.cohesion_velocity_scale {
                velocity_scale = velocity.linvel.length() / config.sheep.max_speed;
            } else {
                velocity_scale = 1.0;
            }
            influences.cohesion_influence = Some(
                correction_to_center * config.flocking.cohesion_scale * velocity_scale / 1000.0
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
                separation * config.flocking.separation_scale / 100000.0
            );
        }
    }
}

pub fn calculate_grazing(
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

pub fn calculate_inertia(
    mut query: Query<(&mut Influences, &Velocity, &ConfigurationSetId), With<Inertia>>,
    config: Res<Configuration>,
) {
    for (
        mut influences,
        velocity,
        config_id
    ) in query.iter_mut() {
        let config_set = config.get_set(config_id);
        let influence = (velocity.linvel / config_set.max_speed) * config.inertia_scale / 100.0;
        influences.inertia_influence = Some(Vec3::from((influence, 0.0)));
    }
}

pub fn calculate_velocity(
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
        if let Some(influence) = influences.inertia_influence {
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

pub fn draw_debug_lines(
    query: Query<(&Transform, &Influences)>,
    mut lines: ResMut<DebugLines>,
    configuration: Res<Configuration>,
) {
    let debug_lines = &configuration.debug_lines;
    if !debug_lines.enable { return; }
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
                DebugLineType::GrazingInfluence => { influences.grazing_influence }
                DebugLineType::InertiaInfluence => { influences.inertia_influence }
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

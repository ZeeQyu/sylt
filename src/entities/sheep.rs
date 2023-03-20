use crate::imports::*;

const NAME: &str = "Sheep";
const Z_INDEX: f32 = 40.0;

#[derive(Default)]
pub struct SheepPlugin;

impl Plugin for SheepPlugin {
    fn build(&self, app: &mut App) {
        app.add_yoleck_handler({
            YoleckTypeHandler::<EditorSheep>::new(NAME)
                .populate_with(populate)
                .edit_with(edit)
                .with(yoleck_vpeol_position_edit_adapter(|data: &mut EditorSheep| {
                    YoleckVpeolTransform2dProjection {
                        translation: &mut data.position,
                    }
                }))
        });
    }
}

#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct EditorSheep {
    #[serde(default)]
    pub position: Vec2,
}

fn populate(mut populate: YoleckPopulate<EditorSheep>, configuration: Res<Configuration>) {
    populate.populate(|_ctx, data, mut commands| {
        commands.insert(SheepBundle::new(&configuration.animation.sheep, data.position));
    });
}

fn edit(mut edit: YoleckEdit<EditorSheep>, mut commands: Commands, mut writer: EventWriter<YoleckEditorEvent>, mut yoleck: ResMut<YoleckState>) {
    edit.edit(|_ctx, data, ui| {
        if ui.add(egui::Button::new("Dolly!")).clicked() {
            let value = serde_json::to_value(EditorSheep { position: data.position + Vec2::splat(20.0) }).unwrap();
            create_editor_object(&mut commands, &mut writer, &mut yoleck, NAME, value);
        }
    });
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
    counts_toward_goal: zone::CountsTowardGoal,
}

impl SheepBundle {
    pub fn new(config_set: &AnimationSheet, position: Vec2) -> Self {
        let mut actor = Actor::new(config_set, position.extend(Z_INDEX), Collider::ball(13.0));
        actor.animation_bundle.animation_timer.0.set_elapsed(
            Duration::from_secs_f32(rand::random::<f32>() * 1.0)
        );
        SheepBundle {
            actor,
            flocking: Flocking::default(),
            grazing: Grazing {
                current_direction: None,
                time_left: (rand::random::<f32>() * 5.0) + 0.0,
            },
            runner: Runner::default(),
            name: Name::new(NAME),
            config_set_id: ConfigurationSetId::Sheep,
            inertia: Inertia::default(),
            counts_toward_goal: zone::CountsTowardGoal,
        }
    }
}

#[derive(Component, Default)]
pub struct Runner {
    pub direction: Vec3,
    pub magnitude: f32,
}

pub fn run_from_players(
    player_query: Query<&GlobalTransform, With<player::PlayerInput>>,
    mut runner_query: Query<(&mut Runner, &mut Influences, &GlobalTransform), Without<player::PlayerInput>>,
    config: Res<Configuration>,
    time: Res<Time>,
) {
    for player_transform in player_query.iter() {
        let player_position = player_transform.translation();
        for (
            mut runner,
            mut influences,
            runner_transform,
        ) in runner_query.iter_mut() {
            let runner_position = runner_transform.translation();
            let mut runner: &mut Runner = &mut runner;

            let scare_distance = config.runner.scare_distance;
            if runner_position.distance(player_position) < scare_distance {
                runner.direction = (runner_position - player_position).normalize_or_zero();
                // runner.direction = (*player_position - *runner_position).normalize_or_zero();
                if runner.magnitude < 0.5 {
                    runner.magnitude = 0.5
                } else {
                    runner.magnitude = f32::min(runner.magnitude + time.delta().as_secs_f32() * 1.0, 1.0);
                }
            } else {
                runner.magnitude -= time.delta().as_secs_f32() * 1.0;
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
}

#[derive(Component, Default)]
pub struct Flocking {
    pub alignment_values: Vec<NeighbourPair>,
    pub cohesion_positions: Vec<Vec3>,
    pub separation_positions: Vec<Vec3>,
}

pub struct NeighbourPair {
    position: Vec3,
    velocity: Vec3,
}

/// Uses the velocity of last frame to find neighbour velocities for flocking behaviour
pub fn find_flocking_neighbours(
    mut query: Query<(Entity, &GlobalTransform, &mut Flocking), With<Velocity>>,
    other_query: Query<(Entity, &GlobalTransform, &Velocity), With<Flocking>>,
    config: Res<Configuration>,
) {
    let config = &config.flocking;
    if config.alignment_enabled || config.cohesion_enabled || config.separation_enabled {
        for (entity, current_transform, mut current_flocking) in query.iter_mut() {
            current_flocking.alignment_values.clear();
            current_flocking.cohesion_positions.clear();
            current_flocking.separation_positions.clear();
            for (other_entity, other_transform, other_velocity) in other_query.iter() {
                if entity == other_entity { continue; }
                let distance = current_transform.translation().distance(other_transform.translation());
                if distance < config.alignment_distance && config.alignment_enabled {
                    current_flocking.alignment_values.push(NeighbourPair {
                        position: other_transform.translation(),
                        velocity: Vec3::from((other_velocity.linvel, 0.0)),
                    });
                }
                if distance < config.cohesion_distance && config.cohesion_enabled {
                    current_flocking.cohesion_positions.push(other_transform.translation());
                }
                if distance < config.separation_distance && config.separation_enabled {
                    current_flocking.separation_positions.push(other_transform.translation());
                }
            }
        }
    }
}

pub fn calculate_flocking(
    mut query: Query<(&mut Influences, &Flocking, &GlobalTransform, &Velocity)>,
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
                let distance = position.distance(transform.translation());
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
            let correction_to_center = cohesion_center - transform.translation();
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
                let to_neighbour: Vec3 = *position - transform.translation();
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

#[derive(Component, Default)]
pub struct Grazing {
    pub current_direction: Option<Vec3>,
    pub time_left: f32,
}

pub fn calculate_grazing(
    mut query: Query<(&mut Influences, &mut Grazing)>,
    config: Res<Configuration>,
    time: Res<Time>,
) {
    for (mut influences, mut grazing) in query.iter_mut() {
        grazing.time_left -= time.delta().as_secs_f32();
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

#[derive(Component, Default)]
pub struct Inertia();


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


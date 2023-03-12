use crate::imports::*;
use crate::imports::sheep::find_flocking_neighbours;

#[derive(Default)]
pub struct MotionPlugin;

impl Plugin for MotionPlugin {
    fn build(&self, app: &mut App) {
        app.configure_set(MotionSet::Prep.before(MotionSet::Main));
        app.configure_set(MotionSet::Main.before(MotionSet::Apply));
        app.configure_sets(
            (
                MotionSet::Prep,
                MotionSet::Main,
                MotionSet::Apply
            ).in_set(GameSet::Motion)
        );
        app.add_systems(
            (
                find_flocking_neighbours,
                reset_influences
            ).in_set(MotionSet::Prep)
        );
        app.add_systems(
            (
                player::apply_player_input,
                sheep::run_from_players,
                sheep::calculate_flocking,
                sheep::calculate_grazing,
                sheep::calculate_inertia,
            ).in_set(MotionSet::Main)
        );
        app.add_systems(
            (
                calculate_velocity,
                draw_debug_lines,
            ).in_set(MotionSet::Apply)
        );
    }
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
enum MotionSet {
    Prep,
    Main,
    Apply,
}


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

pub fn reset_influences(mut query: Query<&mut Influences>) {
    for mut influence in query.iter_mut() {
        *influence = Influences::default();
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
    query: Query<(&GlobalTransform, &Influences)>,
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
                    transform.translation() + Vec3::Z,
                    transform.translation() + influence * line_graphics_scale + Vec3::Z + offset,
                    0.0,
                    color,
                );
            }
        }
    }
}

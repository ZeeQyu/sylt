use crate::imports::*;

const NAME: &str = "GoalZone";
const Z_INDEX: f32 = 18.0;
const Z_INDEX_TEXT_OFFSET: f32 = 0.1; // Should be higher than square

#[derive(Default)]
pub struct GoalZonePlugin;

impl Plugin for GoalZonePlugin {
    fn build(&self, app: &mut App) {
        app.add_yoleck_handler({
            YoleckTypeHandler::<EditorGoalZone>::new(NAME)
                .populate_with(populate)
                .edit_with(edit)
                .with(yoleck_vpeol_position_edit_adapter(|data: &mut EditorGoalZone| {
                    YoleckVpeolTransform2dProjection {
                        translation: &mut data.position,
                    }
                }))
        });
        app.add_system(update_goal_zones);
    }
}

#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
struct EditorGoalZone {
    #[serde(default)]
    position: Vec2,
    #[serde(default = "default_extents")]
    extents: Vec2,
    #[serde(default = "default_target")]
    target: usize,
    #[serde(default = "default_text_size")]
    text_size: f32,
}

fn default_extents() -> Vec2 { Vec2::new(400.0, 300.0) }

fn default_target() -> usize { 30 }

fn default_text_size() -> f32 { 1.0 }

fn populate(
    mut populate: YoleckPopulate<EditorGoalZone>,
    config: Res<Configuration>,
) {
    populate.populate(|_ctx, data, mut commands| {
        commands.despawn_descendants();
        commands.insert((
            YoleckWillContainClickableChildren,
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgba_u8(128, 128, 128, 50),
                    custom_size: Some(data.extents * GLOBAL_TEXTURE_SCALE),
                    ..default()
                },
                transform: Transform::from_translation(data.position.extend(Z_INDEX)),
                global_transform: Transform::from_translation(data.position.extend(Z_INDEX)).into(),
                ..default()
            },
            Collider::cuboid(data.extents.x, data.extents.y),
            Sensor,
            GoalZone { target: data.target },
        )).with_children(|commands| {
            commands.spawn(
                Text2dBundle {
                    text: Text::from_section(
                        String::from(format!("?/{}", data.target)),
                        TextStyle {
                            font: config.global_assets.font.clone().expect("Font should be loaded"),
                            font_size: 72.0,
                            color: Color::BLACK,
                        },
                    ).with_alignment(TextAlignment::CENTER),
                    transform: Transform {
                        // scale: Vec3::new(data.scale, data.scale, Z_INDEX),
                        translation: Vec3::Z * (Z_INDEX_TEXT_OFFSET),
                        scale: Vec3::splat(data.text_size),
                        ..default()
                    },
                    ..default()
                }
            );
        });
    });
}

fn edit(
    mut edit: YoleckEdit<EditorGoalZone>,
) {
    edit.edit(|_ctx, data, ui| {
        ui.horizontal(|ui| {
            ui.add(egui::Label::new("width: "));
            ui.add(egui::DragValue::new(&mut data.extents.x));
            ui.add(egui::Label::new("height: "));
            ui.add(egui::DragValue::new(&mut data.extents.y));
        });
        ui.horizontal(|ui| {
            ui.add(egui::Label::new("target number of sheep: "));
            ui.add(egui::DragValue::new(&mut data.target));
        });
        ui.add(egui::Slider::new(&mut data.text_size, 0.01..=5.0).logarithmic(true));
    });
}

#[derive(Component)]
struct GoalZone {
    target: usize,
}

#[derive(Component)]
pub struct CountsTowardGoal;

fn update_goal_zones(
    zone_query: Query<(Entity, &Children, &GoalZone)>,
    mut text_query: Query<&mut Text>,
    sheep_query: Query<Entity, (With<Collider>, With<CountsTowardGoal>)>,
    // game_mode: Res<CurrentState<GameState>>,
    rapier_context: Res<RapierContext>,
) {
    for (zone_entity, zone_children, goal_zone) in zone_query.iter() {
        let target_num_sheep = goal_zone.target;
        let mut num_sheep = 0;
        for sheep_entity in sheep_query.iter() {
            if rapier_context.intersection_pair(zone_entity, sheep_entity) == Some(true) {
                num_sheep += 1;
            }
        }
        for child in zone_children.iter() {
            if let Ok(mut text) = text_query.get_mut(*child) {
                for mut section in text.sections.iter_mut() {
                    section.value = format!("{num_sheep}/{target_num_sheep}");
                }
            }
        }
    }
}


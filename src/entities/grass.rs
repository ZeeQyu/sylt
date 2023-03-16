use crate::imports::*;

#[derive(Default)]
pub struct GrassPlugin;

const NAME: &str = "Grass";
const Z_INDEX: f32 = 5.0;

impl Plugin for GrassPlugin {
    fn build(&self, app: &mut App) {
        app.add_yoleck_handler({
            YoleckTypeHandler::<EditorGrass>::new(NAME)
                .populate_with(populate)
                .edit_with(edit)
        });
    }
}

#[derive(Bundle)]
pub struct GrassBundle {
    animation_bundle: AnimationBundle,
    name: Name,
    config_set_id: ConfigurationSetId,
}

impl GrassBundle {
    pub fn new(config_set: &AnimationSheet, position: Vec2) -> Self {
        GrassBundle {
            animation_bundle: AnimationBundle::from(config_set, position.extend(Z_INDEX)),
            name: Name::new(NAME),
            config_set_id: ConfigurationSetId::Grass,
        }
    }
}

#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
struct EditorGrass {
    #[serde(default = "default_spawned")]
    num_positions: usize,
    #[serde(default = "default_extents")]
    extents: Vec2,
    #[serde(default = "default_entities")]
    positions: Vec<Vec2>,
}

fn default_spawned() -> usize { 300 }

fn default_extents() -> Vec2 { Vec2::new(1000.0, 1000.0) }

fn default_entities() -> Vec<Vec2> {
    generate_positions(default_spawned(), default_extents())
}
fn generate_positions(num: usize, extents: Vec2) -> Vec<Vec2> {
    let distribution_x = Uniform::new(-extents.x, extents.x);
    let distribution_y = Uniform::new(-extents.y, extents.y);
    let mut positions = Vec::new();
    for _ in 0..num {
        let x = distribution_x.sample(&mut rand::thread_rng());
        let y = distribution_y.sample(&mut rand::thread_rng());
        positions.push(Vec2::new(x, y));
    }
    positions
}

fn populate(
    mut populate: YoleckPopulate<EditorGrass>,
    configuration: Res<Configuration>,
) {
    populate.populate(|_ctx, data, mut commands| {
        commands.despawn_descendants();
        commands.insert((
            TransformBundle::from_transform(Transform::default()),
            ComputedVisibility::default(),
            Visibility::default(),
        ));
        commands.with_children(|commands| {
            for position in data.positions.iter() {
                commands.spawn(GrassBundle::new(
                    &configuration.animation.grass,
                    *position,
                ));
            }
        });
    });
}

fn edit(mut edit: YoleckEdit<EditorGrass>) {
    edit.edit(|_ctx, data, ui| {
        ui.horizontal(|ui| {
            ui.add(egui::Label::new("Num grass instances"));
            ui.add(egui::DragValue::new(&mut data.num_positions));
        });
        ui.horizontal(|ui| {
            ui.add(egui::Label::new("Extents"));
            ui.add(egui::Label::new("x: "));
            ui.add(egui::DragValue::new(&mut data.extents.x));
            ui.add(egui::Label::new("y: "));
            ui.add(egui::DragValue::new(&mut data.extents.y));
        });
        if ui.add(egui::Button::new("Regenerate")).clicked() {
            data.positions = generate_positions(data.num_positions, data.extents);
        }
        ui.collapsing("Individual grass elements", |ui| {
            egui::ScrollArea::vertical().max_height(400.0).show(ui, |ui| {
                for position in data.positions.iter_mut() {
                    ui.horizontal(|ui| {
                        ui.add(egui::Label::new("x: "));
                        ui.add(egui::DragValue::new(&mut position.x));
                        ui.add(egui::Label::new("y: "));
                        ui.add(egui::DragValue::new(&mut position.y));
                    });
                }
            });
        });
    });
}


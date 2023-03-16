use crate::imports::*;

const NAME: &str = "Food";
const Z_INDEX: f32 = 18.0;

#[derive(Default)]
pub struct FoodPlugin;

impl Plugin for FoodPlugin {
    fn build(&self, app: &mut App) {
        app.add_yoleck_handler({
            YoleckTypeHandler::<EditorFood>::new(NAME)
                .populate_with(populate)
                .edit_with(edit)
                .with(yoleck_vpeol_position_edit_adapter(|data: &mut EditorFood| {
                    YoleckVpeolTransform2dProjection {
                        translation: &mut data.position,
                    }
                }))
        });
    }
}

#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
struct EditorFood {
    #[serde(default)]
    position: Vec2,
    #[serde(default = "default_strength")]
    strength: f32,

}
fn default_strength() -> f32 { 1.0 }

fn populate(
    mut populate: YoleckPopulate<EditorFood>,
    configuration: Res<Configuration>,
) {
    populate.populate(|_ctx, data, mut commands| {
        commands.insert(FoodBundle::new(
            &configuration.animation,
            data,
        ));
    });
}

fn edit(
    mut edit: YoleckEdit<EditorFood>,
) {
    edit.edit(|_ctx, data, ui| {
        ui.add(egui::DragValue::new(&mut data.strength));
    });
}

#[derive(Component)]
struct Food {
    _strength: f32,
}

#[derive(Bundle)]
pub struct FoodBundle {
    animation_bundle: AnimationBundle,
    collider: Collider,
    rigid_body: RigidBody,
    name: Name,
    config_set_id: ConfigurationSetId,
    food: Food,
}

impl FoodBundle {
    fn new(config: &AnimationConfiguration, editor_food: &EditorFood) -> Self {
        let dimension = 13.0;
        FoodBundle {
            animation_bundle: AnimationBundle::from(&config.food, editor_food.position.extend(Z_INDEX)),
            collider: Collider::cuboid(dimension, dimension),
            rigid_body: RigidBody::Fixed,
            name: Name::new(NAME),
            config_set_id: ConfigurationSetId::Food,
            food: Food { _strength: editor_food.strength},
        }
    }
}

#[derive(Component)]
pub struct LikesFood;

// pub fn go_for_food(
//     mut query: Query<>
// )

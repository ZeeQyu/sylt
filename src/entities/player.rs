use crate::imports::*;

const PLAYER_NAME: &str = "Player";

#[derive(Default)]
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_yoleck_handler({
            YoleckTypeHandler::<EditorPlayer>::new(PLAYER_NAME)
                .populate_with(populate_player)
                .with(yoleck_vpeol_position_edit_adapter(|data: &mut EditorPlayer| {
                    YoleckVpeolTransform2dProjection {
                        translation: &mut data.position,
                    }
                }))
        });
        app.add_system_set(ConditionSet::new()
            .run_in_state(GameState::Game)
            .with_system(apply_player_input)
            .into()
        );
    }
}

#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
struct EditorPlayer {
    #[serde(default)]
    position: Vec2,
}

fn populate_player(mut populate: YoleckPopulate<EditorPlayer>, configuration: Res<Configuration>) {
    populate.populate(|_ctx, data, mut commands| {
        commands.insert(PlayerBundle::new(&configuration.animation.player, data.position.extend(0.0)));
    });
}


#[derive(Bundle)]
pub struct PlayerBundle {
    actor: Actor,
    player: PlayerInput,
    name: Name,
    config_set_id: ConfigurationSetId,
}

impl PlayerBundle {
    pub fn new(config_set: &AnimationSheet, position: Vec3) -> Self {
        PlayerBundle {
            actor: Actor::new(config_set, position, Collider::ball(15.0)),
            player: PlayerInput {},
            name: Name::new("Player"),
            config_set_id: ConfigurationSetId::Player,
        }
    }
}

#[derive(Component)]
pub struct PlayerInput {}

pub fn apply_player_input(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Influences, With<PlayerInput>>,
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
    for mut influences in query.iter_mut() {
        influences.player_input_influence = Some(direction);
    }
}

use crate::imports::*;

const NAME: &str = "GameRules";

#[derive(Default)]
pub struct GameRulesPlugin;

impl Plugin for GameRulesPlugin {
    fn build(&self, app: &mut App) {
        app.add_yoleck_handler({
            YoleckTypeHandler::<GameRules>::new(NAME)
                .populate_with(populate)
                .edit_with(edit)
        });
    }
}

#[derive(Component, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
struct GameRules {
    #[serde(default)]
    victory_condition: VictoryCondition,
}

#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize, Default)]
enum VictoryCondition {
    #[default]
    AllGoalZones,
    AnyGoalZones,
}


fn populate(
    mut populate: YoleckPopulate<GameRules>,
) {
    populate.populate(|_ctx, data, mut commands| {
        commands.insert((
            data.clone(),
        ));
    });
}

fn edit(
    mut edit: YoleckEdit<GameRules>,
) {
    edit.edit(|_ctx, data, ui| {
        ui.horizontal(|ui| {
            {
                let value = VictoryCondition::AllGoalZones;
                if ui.add_enabled(data.victory_condition != value, egui::Button::new("AllGoalZones")).clicked() {
                    data.victory_condition = value
                }
            }
            {
                let value = VictoryCondition::AnyGoalZones;
                if ui.add_enabled(data.victory_condition != value, egui::Button::new("AnyGoalZones")).clicked() {
                    data.victory_condition = value
                }
            }
        });
    });
}


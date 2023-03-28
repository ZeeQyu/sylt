use crate::imports::*;
use crate::levels::LevelEvent;
use crate::levels::LevelEvent::LoadNextLevel;

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
        app.add_event::<GameRulesCommand>();
        app.add_system(handle_game_rules.run_in_state(GameState::Game));
    }
}

#[derive(Debug)]
pub enum GameRulesCommand {
    CheckSheepWin {
        all_zones_done: bool,
        any_zones_done: bool,
    }
}

#[derive(Component, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GameRules {
    #[serde(default)]
    pub victory_condition: VictoryCondition,
}

#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize, Default)]
pub enum VictoryCondition {
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

pub fn handle_game_rules(
    mut event_reader: EventReader<GameRulesCommand>,
    game_rules_query: Query<&game_rules::GameRules>,
    mut event_writer: EventWriter<LevelEvent>,
) {
    let victory_conditions = if let Ok(game_rules) = game_rules_query.get_single() {
        &game_rules.victory_condition
    } else {
        &VictoryCondition::AllGoalZones
    };
    for command in event_reader.iter() {
        println!("{:?}", command);
        match command {
            GameRulesCommand::CheckSheepWin { all_zones_done, any_zones_done } => {
                match victory_conditions {
                    VictoryCondition::AllGoalZones => {
                        if *all_zones_done {
                            event_writer.send(LoadNextLevel);
                        }
                    }
                    VictoryCondition::AnyGoalZones => {
                        if *any_zones_done {
                            event_writer.send(LoadNextLevel);
                        }
                    }
                }
            }
        }
    }
}

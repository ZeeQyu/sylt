use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use bevy_yoleck::vpeol_2d::{yoleck_vpeol_position_edit_adapter, YoleckVpeolTransform2dProjection};
use bevy_yoleck::{egui, YoleckEdit, YoleckEditorEvent, YoleckEditorState, YoleckEntryHeader, YoleckExtForApp, YoleckPopulate, YoleckRawEntry, YoleckState, YoleckTypeHandler};
use bevy_yoleck::vpeol::YoleckWillContainClickableChildren;
use iyes_loopless::prelude::*;
use rand::distributions::Distribution;
use rand_distr::Normal;
use crate::{GameState, motion, spawning};
use serde::Serialize;
use serde::Deserialize;

#[derive(Default)]
pub struct EditorPlugin;

const PLAYER_NAME: &str = "Player";
const SHEEP_NAME: &str = "Sheep";
const SHEEP_CLUSTER_NAME: &str = "SheepCluster";

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app
            // .add_plugin(bevy_yoleck::bevy_egui::EguiPlugin)
            .add_plugin(bevy_yoleck::YoleckPluginForEditor)
            .add_plugin(bevy_yoleck::vpeol_2d::YoleckVpeol2dPlugin)
            .add_plugin(SyncWithEditorState {
                when_editor: GameState::Editor,
                when_game: GameState::Game,
            })
            .add_system(immobilize_physics_bodies.run_in_state(GameState::Editor))
            .add_exit_system(GameState::Editor, remobilize_physics_bodies)
            .add_enter_system(GameState::Editor, show_clusters)
            .add_exit_system(GameState::Editor, hide_clusters)
        ;
        app.add_yoleck_handler({
            YoleckTypeHandler::<EditorPlayer>::new(PLAYER_NAME)
                .populate_with(populate_player)
                .with(yoleck_vpeol_position_edit_adapter(|data: &mut EditorPlayer| {
                    YoleckVpeolTransform2dProjection {
                        translation: &mut data.position,
                    }
                }))
        });
        app.add_yoleck_handler({
            YoleckTypeHandler::<EditorSheep>::new(SHEEP_NAME)
                .populate_with(populate_sheep)
                .edit_with(edit_sheep)
                .with(yoleck_vpeol_position_edit_adapter(|data: &mut EditorSheep| {
                    YoleckVpeolTransform2dProjection {
                        translation: &mut data.position,
                    }
                }))
        });
        app.add_yoleck_handler({
            YoleckTypeHandler::<EditorSheepCluster>::new(SHEEP_CLUSTER_NAME)
                .populate_with(populate_sheep_cluster)
                .edit_with(edit_sheep_cluster)
                .with(yoleck_vpeol_position_edit_adapter(|data: &mut EditorSheepCluster| {
                    YoleckVpeolTransform2dProjection {
                        translation: &mut data.position,
                    }
                }))
        });
    }
}

pub struct SyncWithEditorState<T>
    where
        T: 'static + Sync + Send + std::fmt::Debug + Clone + std::cmp::Eq + std::hash::Hash,
{
    pub when_editor: T,
    pub when_game: T,
}

impl<T> Plugin for SyncWithEditorState<T>
    where
        T: 'static + Sync + Send + std::fmt::Debug + Clone + std::cmp::Eq + std::hash::Hash,
{
    fn build(&self, app: &mut App) {
        let when_editor = self.when_editor.clone();
        app.add_loopless_state(when_editor.clone());
        let when_game = self.when_game.clone();
        app.add_system(
            move |editor_state: Res<State<bevy_yoleck::YoleckEditorState>>, mut commands: Commands| {
                let next_state = match editor_state.current() {
                    YoleckEditorState::EditorActive => when_editor.clone(),
                    YoleckEditorState::GameActive => when_game.clone(),
                };
                commands.insert_resource(NextState(next_state));
            },
        );
    }
}

#[derive(Component)]
struct ImmobilizedPhysicsBody {
    rigid_body: RigidBody,
    collider: Option<Collider>,
}

fn immobilize_physics_bodies(query: Query<(Entity, &RigidBody, Option<&Collider>)>, mut commands: Commands) {
    for (entity, rigid_body, collider) in query.iter() {
        commands.entity(entity).insert(ImmobilizedPhysicsBody {
            rigid_body: rigid_body.clone(),
            collider: collider.cloned(),
        }).remove::<RigidBody>().remove::<Collider>();
    }
}

fn remobilize_physics_bodies(query: Query<(Entity, &ImmobilizedPhysicsBody)>, mut commands: Commands) {
    for (entity, immobilized) in query.iter() {
        if let Some(collider) = &immobilized.collider {
            commands.entity(entity)
                .insert(immobilized.rigid_body)
                .insert(collider.clone())
                .remove::<ImmobilizedPhysicsBody>();
        } else {
            commands.entity(entity)
                .insert(immobilized.rigid_body)
                .remove::<ImmobilizedPhysicsBody>();
        }
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
struct EditorPlayer {
    #[serde(default)]
    position: Vec2,
}
fn populate_player(mut populate: YoleckPopulate<EditorPlayer>, configuration: Res<motion::Configuration>) {
    populate.populate(|_ctx, data, mut commands| {
        commands.insert(spawning::PlayerBundle::new(&configuration.animation.player, data.position.extend(0.0)));
    });
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
struct EditorSheep {
    #[serde(default)]
    position: Vec2,
}

fn populate_sheep(mut populate: YoleckPopulate<EditorSheep>, configuration: Res<motion::Configuration>) {
    populate.populate(|_ctx, data, mut commands| {
        commands.insert(spawning::SheepBundle::new(&configuration.animation.sheep, data.position.extend(0.0)));
    });
}

fn edit_sheep(mut edit: YoleckEdit<EditorSheep>, mut commands: Commands, mut writer: EventWriter<YoleckEditorEvent>, mut yoleck: ResMut<YoleckState>) {
    edit.edit(|_ctx, data, ui| {
        if ui.add(egui::Button::new("Dolly!")).clicked() {
            let cmd = commands.spawn(YoleckRawEntry {
                header: YoleckEntryHeader {
                    type_name: String::from(SHEEP_NAME),
                    name: String::from(""),
                },
                data: serde_json::to_value(EditorSheep { position: data.position + Vec2::splat(20.0) }).unwrap(),
                // data: serde_json::Value::Object(serde_json::Serializer:: Default::default()),
                // data: serde_json::Value::Object(EditorSheep { position: data.position + Vec2::splat(5.0) }),
            });
            writer.send(YoleckEditorEvent::EntitySelected(cmd.id()));
            yoleck.entity_being_edited = Some(cmd.id());
            yoleck.level_needs_saving = true;
        }
    });
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
struct EditorSheepCluster {
    #[serde(default)]
    position: Vec2,
    #[serde(default)]
    std_dev_radius: f32,
    #[serde(default)]
    num_sheep: usize,
    #[serde(default)]
    sheep: Vec<EditorSheep>,
}

fn populate_sheep_cluster(
    mut populate: YoleckPopulate<EditorSheepCluster>,
    configuration: Res<motion::Configuration>,
    state: Res<CurrentState<GameState>>,
) {
    populate.populate(|_ctx, data, mut commands| {
        commands.despawn_descendants();
        commands.insert((
            TransformBundle::from_transform(Transform::from_translation(data.position.extend(0.0))),
            ComputedVisibility::default(),
            Visibility::default(),
            YoleckWillContainClickableChildren,
        ));
        commands.with_children(|commands| {
            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::PURPLE,
                        custom_size: Some(Vec2::splat(30.0)),
                        ..default()
                    },
                    transform: Transform::default(),
                    global_transform: Transform::default().into(),
                    visibility: if state.0 == GameState::Editor { Visibility::VISIBLE } else { Visibility::INVISIBLE },
                    ..default()
                },
                IsCluster,
            ));
            for sheep in data.sheep.iter() {
                commands.spawn(spawning::SheepBundle::new(&configuration.animation.sheep, sheep.position.extend(0.0)));
            };
        });
    });
}

fn edit_sheep_cluster(mut edit: YoleckEdit<EditorSheepCluster>) {
    edit.edit(|_ctx, data, ui| {
        ui.horizontal(|ui| {
            ui.add(egui::Label::new("Num sheep"));
            ui.add(egui::DragValue::new(&mut data.num_sheep));
        });
        ui.horizontal(|ui| {
            ui.add(egui::Label::new("Std dev radius"));
            ui.add(egui::DragValue::new(&mut data.std_dev_radius));
        });
        if ui.add(egui::Button::new("Regenerate")).clicked() {
            data.sheep.clear();
            let distribution = Normal::new(0.0, data.std_dev_radius).unwrap();
            for _ in 0..data.num_sheep {
                let x = distribution.sample(&mut rand::thread_rng());
                let y = distribution.sample(&mut rand::thread_rng());
                data.sheep.push(EditorSheep { position: Vec2::new(x, y) });
            }
        }
        ui.collapsing("Individual Sheep", |ui| {
            egui::ScrollArea::vertical().max_height(400.0).show(ui, |ui| {
                for sheep in data.sheep.iter_mut() {
                    ui.horizontal(|ui| {
                        ui.add(egui::Label::new("x: "));
                        ui.add(egui::DragValue::new(&mut sheep.position.x));
                        ui.add(egui::Label::new("y: "));
                        ui.add(egui::DragValue::new(&mut sheep.position.y));
                    });
                }
            });
        });
    });
}

#[derive(Component)]
struct IsCluster;

fn hide_clusters(mut query: Query<&mut Visibility, With<IsCluster>>) {
    for mut visibility in query.iter_mut() {
        visibility.is_visible = false;
    }
}

fn show_clusters(mut query: Query<&mut Visibility, With<IsCluster>>) {
    for mut visibility in query.iter_mut() {
        visibility.is_visible = true;
    }
}

// fn edit_rectangle(mut edit: YoleckEdit<Rectangle>, mut commands: Commands) {

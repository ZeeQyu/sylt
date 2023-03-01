use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use bevy_yoleck::vpeol_2d::{yoleck_vpeol_position_edit_adapter, YoleckVpeolTransform2dProjection};
use bevy_yoleck::{egui, YoleckEdit, YoleckEditorEvent, YoleckEditorState, YoleckEntryHeader, YoleckExtForApp, YoleckPopulate, YoleckRawEntry, YoleckState, YoleckTypeHandler};
use bevy_yoleck::vpeol::YoleckWillContainClickableChildren;
use iyes_loopless::prelude::*;
use rand::distributions::Distribution;
use rand_distr::Normal;
use crate::{GameState, motion, spawning, animation::GLOBAL_TEXTURE_SCALE};
use serde::Serialize;
use serde::Deserialize;
use serde_json::Value;
use crate::animation::AnimationConfiguration;
use crate::spawning::FenceOrientation;

#[derive(Default)]
pub struct EditorPlugin;

const PLAYER_NAME: &str = "Player";
const SHEEP_NAME: &str = "Sheep";
const SHEEP_CLUSTER_NAME: &str = "SheepCluster";
const FENCE_NAME: &str = "Fence";

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
        app.add_yoleck_handler({
            YoleckTypeHandler::<EditorFence>::new(FENCE_NAME)
                .populate_with(populate_fence)
                .edit_with(edit_fence)
                .with(yoleck_vpeol_position_edit_adapter(|data: &mut EditorFence| {
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
            let value = serde_json::to_value(EditorSheep { position: data.position + Vec2::splat(20.0) }).unwrap();
            create_editor_object(&mut commands, &mut writer, &mut yoleck, SHEEP_NAME, value);
        }
    });
}

fn create_editor_object(commands: &mut Commands, writer: &mut EventWriter<YoleckEditorEvent>, yoleck: &mut ResMut<YoleckState>, type_name: &str, value: Value) {
    let cmd = commands.spawn(YoleckRawEntry {
        header: YoleckEntryHeader {
            type_name: String::from(type_name),
            name: String::from(""),
        },
        data: value,
    });
    writer.send(YoleckEditorEvent::EntitySelected(cmd.id()));
    yoleck.entity_being_edited = Some(cmd.id());
    yoleck.level_needs_saving = true;
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

#[derive(Clone, PartialEq, Serialize, Deserialize)]
struct EditorFence {
    #[serde(default)]
    position: Vec2,
    #[serde(default)]
    orientation: FenceOrientation,
    #[serde(default)]
    section_length: f32,
}

fn populate_fence(
    mut populate: YoleckPopulate<EditorFence>,
    configuration: Res<motion::Configuration>,
) {
    populate.populate(|_ctx, data, mut commands| {
        let (axis, texture_length) = fence_axis_and_length(&configuration.animation, &data.orientation);
        commands.despawn_descendants();
        commands.insert((
            TransformBundle::from_transform(Transform::from_translation(data.position.extend(0.0))),
            ComputedVisibility::default(),
            Visibility::default(),
            YoleckWillContainClickableChildren,
        ));
        commands.with_children(|commands| {
            let num_sections = (data.section_length / texture_length) as u32 + 1;
            for i in 0..num_sections {
                let position = i as f32 * axis * texture_length - 4.0 * i as f32 * axis + Vec3::Z * i as f32;
                commands.spawn(spawning::FenceBundle::new(
                    &configuration.animation,
                    &data.orientation,
                    position,
                ));
            }
        });
    });
}

fn edit_fence(
    mut edit: YoleckEdit<EditorFence>,
    configuration: Res<motion::Configuration>,
    mut commands: Commands,
    mut writer: EventWriter<YoleckEditorEvent>,
    mut yoleck: ResMut<YoleckState>,
) {
    edit.edit(|ctx, data, ui| {
        if ui.add(egui::Button::new("Spawn copy")).clicked() {
            let offset_axis = match data.orientation {
                FenceOrientation::Horizontal => -Vec2::Y,
                FenceOrientation::Vertical => Vec2::X,
            };
            let value = serde_json::to_value(EditorFence { position: data.position + offset_axis * 20.0, ..data.clone() }).unwrap();
            create_editor_object(&mut commands, &mut writer, &mut yoleck, FENCE_NAME, value);
        }
        ui.horizontal(|ui| {
            {
                let orientation = FenceOrientation::Horizontal;
                if ui.add_enabled(data.orientation != orientation, egui::Button::new("Horizontal")).clicked() {
                    data.orientation = orientation
                }
            }
            {
                let orientation = FenceOrientation::Vertical;
                if ui.add_enabled(data.orientation != orientation, egui::Button::new("Vertical")).clicked() {
                    data.orientation = orientation
                }
            }
        });
        let (axis, _texture_length) = fence_axis_and_length(&configuration.animation, &data.orientation);
        if data.section_length < 0.0 {
            data.section_length = 0.0;
        }
        let mut knob = ctx.knob(&mut commands, "knob");
        let knob_position = data.position.extend(1.0) + (data.section_length * axis);
        knob.cmd.insert(SpriteBundle {
            sprite: Sprite {
                color: Color::RED,
                custom_size: Some(Vec2::splat(15.0)),
                ..default()
            },
            transform: Transform::from_translation(knob_position),
            global_transform: Transform::from_translation(knob_position).into(),
            ..default()
        });
        if let Some(extend_to) = knob.get_passed_data::<Vec2>() {
            match data.orientation {
                FenceOrientation::Horizontal => {
                    data.section_length = axis.signum().x * (extend_to.x - data.position.x);
                }
                FenceOrientation::Vertical => {
                    data.section_length = axis.signum().y * (extend_to.y - data.position.y);
                }
            }
        }
    });
}

fn fence_axis_and_length(config: &AnimationConfiguration, orientation: &FenceOrientation) -> (Vec3, f32) {
    match orientation {
        FenceOrientation::Horizontal => (Vec3::X, config.fence_horizontal.texture_size.x * GLOBAL_TEXTURE_SCALE),
        FenceOrientation::Vertical => (-Vec3::Y, config.fence_vertical.texture_size.y * GLOBAL_TEXTURE_SCALE),
    }
}


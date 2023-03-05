use crate::imports::*;

const SHEEP_CLUSTER_NAME: &str = "SheepCluster";

#[derive(Default)]
pub struct SheepClusterPlugin;

impl Plugin for SheepClusterPlugin {
    fn build(&self, app: &mut App) {
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
        app.add_enter_system(GameState::Editor, show_clusters);
        app.add_exit_system(GameState::Editor, hide_clusters);
    }
}

#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
struct EditorSheepCluster {
    #[serde(default)]
    position: Vec2,
    #[serde(default)]
    std_dev_radius: f32,
    #[serde(default)]
    num_sheep: usize,
    #[serde(default)]
    sheep: Vec<sheep::EditorSheep>,
}

fn populate_sheep_cluster(
    mut populate: YoleckPopulate<EditorSheepCluster>,
    configuration: Res<Configuration>,
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
                commands.spawn(sheep::SheepBundle::new(&configuration.animation.sheep, sheep.position.extend(0.0)));
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
                data.sheep.push(sheep::EditorSheep { position: Vec2::new(x, y) });
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


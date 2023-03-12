use crate::imports::*;

const FENCE_NAME: &str = "Fence";

#[derive(Default)]
pub struct FencePlugin;

impl Plugin for FencePlugin {
    fn build(&self, app: &mut App) {
        app.add_yoleck_handler({
            YoleckTypeHandler::<EditorFence>::new(FENCE_NAME)
                .populate_with(populate_fence)
                .edit_with(edit_fence)
                .with(vpeol_position_edit_adapter(|data: &mut EditorFence| {
                    VpeolTransform2dProjection {
                        translation: &mut data.position,
                    }
                }))
        });
    }
}

#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
struct EditorFence {
    #[serde(default)]
    position: Vec2,
    #[serde(default)]
    orientation: FenceOrientation,
    #[serde(default)]
    section_length: f32,
}

#[derive(Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum FenceOrientation {
    #[default]
    Horizontal,
    Vertical,
}

fn populate_fence(
    mut populate: YoleckPopulate<EditorFence>,
    configuration: Res<Configuration>,
) {
    populate.populate(|_ctx, data, mut commands| {
        let (axis, texture_length) = fence_axis_and_length(&configuration.animation, &data.orientation);
        commands.despawn_descendants();
        commands.insert((
            TransformBundle::from_transform(Transform::from_translation(data.position.extend(0.0))),
            ComputedVisibility::default(),
            Visibility::default(),
            VpeolWillContainClickableChildren,
        ));
        commands.with_children(|commands| {
            let num_sections = (data.section_length / texture_length) as u32 + 1;
            for i in 0..num_sections {
                let position = i as f32 * axis * texture_length - 4.0 * i as f32 * axis + Vec3::Z * i as f32;
                commands.spawn(FenceBundle::new(
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
    configuration: Res<Configuration>,
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

#[derive(Bundle)]
pub struct FenceBundle {
    animation_bundle: AnimationBundle,
    collider: Collider,
    rigid_body: RigidBody,
    name: Name,
    config_set_id: ConfigurationSetId,
}

impl FenceBundle {
    pub fn new(config: &AnimationConfiguration, fence_orientation: &FenceOrientation, position: Vec3) -> Self {
        // let (config_set_id, dimensions)= match fence_orientation {
        //     FenceOrientation::Horizontal => { (ConfigurationSetId::FenceHorizontal, Vec2::new(5.0, 2.0) )}
        //     FenceOrientation::Vertical => { (ConfigurationSetId::FenceVertical, Vec2::new(2.0, 5.0)) }
        // };
        let config_set_id = match fence_orientation {
            FenceOrientation::Horizontal => { ConfigurationSetId::FenceHorizontal }
            FenceOrientation::Vertical => { ConfigurationSetId::FenceVertical }
        };
        let config_set = config.get_set(&config_set_id);
        let dimensions = config_set.texture_size;
        FenceBundle {
            animation_bundle: AnimationBundle::from(config_set, position),
            collider: Collider::cuboid(dimensions.x, dimensions.y),
            rigid_body: RigidBody::Fixed,
            name: Name::new("Fence"),
            config_set_id,
        }
    }
}


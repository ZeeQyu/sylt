use crate::imports::*;

const NAME: &str = "Text";
const Z_INDEX: f32 = 30.0;

#[derive(Default)]
pub struct TextPlugin;

impl Plugin for TextPlugin {
    fn build(&self, app: &mut App) {
        app.add_yoleck_handler({
            YoleckTypeHandler::<EditorText>::new(NAME)
                .populate_with(populate)
                .edit_with(edit)
                .with(yoleck_vpeol_position_edit_adapter(|data: &mut EditorText| {
                    YoleckVpeolTransform2dProjection {
                        translation: &mut data.position,
                    }
                }))
        });
    }
}

#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct EditorText {
    #[serde(default)]
    position: Vec2,
    #[serde(default)]
    text: String,
    #[serde(default = "default_scale")]
    scale: f32,
}

fn default_scale() -> f32 { 0.3 }

fn populate(mut populate: YoleckPopulate<EditorText>, config: Res<Configuration>) {
    populate.populate(|ctx, data, mut commands| {
        let text = if ctx.is_in_editor() && data.text.trim_start().is_empty() {
            String::from("New text")
        } else {
            data.text.clone()
        };
        commands.insert(Text2dBundle {
            text: Text::from_section(
                text,
                TextStyle {
                    font: config.global_assets.font.clone().expect("Font should be loaded"),
                    font_size: 72.0,
                    color: Color::WHITE,
                },
            ),
            transform: Transform {
                translation: data.position.extend(Z_INDEX),
                rotation: Default::default(),
                scale: Vec3::new(data.scale, data.scale, 1.0),
            },
            ..default()
        });
    });
}

fn edit(mut edit: YoleckEdit<EditorText>) {
    edit.edit(|_ctx, data, ui| {
        ui.label("Text contents:");
        ui.text_edit_multiline(&mut data.text);
        ui.add(egui::Slider::new(&mut data.scale, 0.1..=1.0).logarithmic(true));
    });
}
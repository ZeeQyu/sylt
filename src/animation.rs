use std::ops::Add;
use bevy::prelude::*;
use bevy::time::FixedTimestep;
use bevy_rapier2d::prelude::Velocity;
use bevy_inspector_egui::prelude::*;
use crate::{TIME_STEP, ConfigurationSetId};
use crate::motion::Configuration;
use std::ops::Mul;
use std::time::Duration;

#[derive(Default)]
pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system_set(
                SystemSet::new()
                    .label("animation")
                    .after("motion")
                    .with_run_criteria(FixedTimestep::step(TIME_STEP as f64))
                    .with_system(animate_sprite)
            );
    }
}

impl AnimationConfiguration {
    pub fn new() -> Self {
        Self {
            player: AnimationSet {
                texture_size: Vec2::new(16.0, 16.0),
                scale: Vec2::new(2.0, 2.0),
                run_threshold_fraction: 0.3,
                idle: SingleAnimation {
                    sprite_sheet: String::from("Collie.png"),
                    sprite_sheet_handle: None,
                    animation_interval: 1.0,
                    atlas_tile_columns: 1,
                    atlas_tile_rows: 1,

                },
                running: SingleAnimation {
                    sprite_sheet: String::from("Collie-run-sheet.png"),
                    sprite_sheet_handle: None,
                    animation_interval: 0.3,
                    atlas_tile_columns: 2,
                    atlas_tile_rows: 1,
                },
            },

            sheep: AnimationSet {
                texture_size: Vec2::new(16.0, 16.0),
                scale: Vec2::new(2.0, 2.0),
                run_threshold_fraction: 0.3,
                idle: SingleAnimation {
                    sprite_sheet: String::from("Sheep.png"),
                    sprite_sheet_handle: None,
                    animation_interval: 1.0,
                    atlas_tile_columns: 1,
                    atlas_tile_rows: 1,
                },
                running: SingleAnimation {
                    sprite_sheet: String::from("Sheep-sheet.png"),
                    sprite_sheet_handle: None,
                    animation_interval: 0.3,
                    atlas_tile_columns: 4,
                    atlas_tile_rows: 1,
                },
            },
        }
    }
    fn get_set<'a>(self: &'a Self, id: &ConfigurationSetId) -> &'a AnimationSet {
        match id {
            ConfigurationSetId::Player => {
                &self.player
            }
            ConfigurationSetId::Sheep => {
                &self.sheep
            }
        }
    }
}

#[derive(Reflect, Default, Resource)]
#[reflect(Resource)]
pub struct AnimationConfiguration {
    pub player: AnimationSet,
    pub sheep: AnimationSet,
}

#[derive(Reflect, Default)]
pub struct AnimationSet {
    idle: SingleAnimation,
    running: SingleAnimation,
    run_threshold_fraction: f32,
    texture_size: Vec2,
    scale: Vec2,
}

impl AnimationSet {
    fn get_anim(self: &Self, anim: AnimationType) -> &SingleAnimation {
        match anim {
            AnimationType::Idle => &self.idle,
            AnimationType::Running => &self.running,
        }
    }
}

#[derive(Reflect, Default, InspectorOptions)]
#[reflect(InspectorOptions)]
struct SingleAnimation {
    sprite_sheet: String,
    #[reflect(ignore)]
    sprite_sheet_handle: Option<Handle<TextureAtlas>>,
    animation_interval: f32,
    atlas_tile_columns: usize,
    atlas_tile_rows: usize,
}

#[derive(Component, Copy, Clone, PartialEq)]
enum AnimationType {
    Idle,
    Running,
}

pub fn load_sprite_sheets(asset_server: Res<AssetServer>, texture_atlases: &mut ResMut<Assets<TextureAtlas>>, anim_config: &mut AnimationConfiguration) {
    macro_rules! load {
        ($name:ident, $anim:ident) => {
            let mut single_animation = &mut anim_config.$name.$anim;
            let texture_handle = asset_server.load(&single_animation.sprite_sheet);
            let texture_atlas = TextureAtlas::from_grid(
                texture_handle,
                anim_config.$name.texture_size,
                single_animation.atlas_tile_columns,
                single_animation.atlas_tile_rows,
                None,
                None,
            );
            let texture_atlas_handle = texture_atlases.add(texture_atlas);
            single_animation.sprite_sheet_handle = Some(texture_atlas_handle);
        };
    }
    load!(player, idle);
    load!(player, running);
    load!(sheep, idle);
    load!(sheep, running);
}


#[derive(Bundle)]
pub struct AnimationBundle {
    sprite_sheet: SpriteSheetBundle,
    animation_timer: AnimationTimer,
    indices: AnimationIndices,
    current_animation: AnimationType,
}

impl AnimationBundle {
    pub fn from(config_set: &AnimationSet, position: Vec3) -> Self {
        let num_sprites = config_set.idle.atlas_tile_columns * config_set.idle.atlas_tile_rows;
        let a: Vec2;
        Self {
            sprite_sheet: SpriteSheetBundle {
                texture_atlas: config_set.idle.sprite_sheet_handle.clone().expect(
                    &format!("all sprite sheets should be loaded for the game to run, missing {}", config_set.idle.sprite_sheet)),
                sprite: TextureAtlasSprite {
                    index: 0,
                    custom_size: Some(config_set.texture_size * config_set.scale),
                    ..default()
                },
                transform: Transform::from_translation(position),
                ..default()
            },
            animation_timer: AnimationTimer(Timer::from_seconds(config_set.idle.animation_interval, TimerMode::Repeating)),
            indices: AnimationIndices { first: 0, last: num_sprites - 1 },
            current_animation: AnimationType::Idle,
        }
    }
}

#[derive(Component)]
struct AnimationIndices {
    first: usize,
    last: usize,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(
        &mut TextureAtlasSprite,
        &mut AnimationTimer,
        &mut AnimationType,
        &mut AnimationIndices,
        &ConfigurationSetId,
        Option<&Velocity>,
    )>,
    config: Res<Configuration>,
) {
    for (
        mut sprite,
        mut timer,
        mut current_anim,
        mut indices,
        config_id,
        velocity
    ) in query.iter_mut() {
        let config_set = config.animation.get_set(config_id);
        let new_anim;
        if let Some(velocity) = velocity {
            let speed_fraction = velocity.linvel.length() / config.get_set(config_id).max_speed;
            if speed_fraction > config_set.run_threshold_fraction {
                new_anim = AnimationType::Idle;
            } else {
                new_anim = AnimationType::Running;
            }
            if new_anim != *current_anim {
                let animation = config_set.get_anim(new_anim);
                timer.set_elapsed(Duration::from_secs_f32(0.0));
                timer.set_duration(Duration::from_secs_f32(animation.animation_interval));
                let num_sprites = config_set.idle.atlas_tile_columns * config_set.idle.atlas_tile_rows;
                indices.last = num_sprites - 1;
                *current_anim = new_anim;
                // bundle.texture_atlas = animation.sprite_sheet_handle.clone().expect("all handles should be loaded");
            }
        }
        timer.tick(time.delta());
        if timer.just_finished() {
            sprite.index = if sprite.index == indices.last {
                indices.first
            } else {
                sprite.index + 1
            }
        }
    }
}
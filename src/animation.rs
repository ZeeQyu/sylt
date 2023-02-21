use bevy::prelude::*;
use bevy::time::FixedTimestep;
use bevy_rapier2d::prelude::Velocity;
use bevy_inspector_egui::prelude::*;
use crate::{TIME_STEP, ConfigurationSetId};
use crate::motion::Configuration;
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
                sprite_sheet: String::from("Collie-run-sheet.png"),
                sprite_sheet_handle: None,
                atlas_tile_columns: 2,
                atlas_tile_rows: 1,
                texture_size: Vec2::new(20.0, 16.0),
                scale: Vec2::new(2.0, 2.0),
                snappy_animations: true,
                animation_class: AnimationClass::Actor {
                    run_threshold_fraction: 0.3,
                    flip_threshold_fraction: 0.1,
                    idle: SingleAnimation {
                        animation_interval: 0.3,
                        first_index: 0,
                        last_index: 0,
                    },
                    running: SingleAnimation {
                        animation_interval: 0.15,
                        first_index: 0,
                        last_index: 1,
                    },
                },
            },

            sheep: AnimationSet {
                sprite_sheet: String::from("Sheep-sheet.png"),
                sprite_sheet_handle: None,
                atlas_tile_columns: 4,
                atlas_tile_rows: 1,
                texture_size: Vec2::new(16.0, 16.0),
                scale: Vec2::new(2.0, 2.0),
                snappy_animations: false,
                animation_class: AnimationClass::Actor {
                    run_threshold_fraction: 0.3,
                    flip_threshold_fraction: 0.2,
                    idle: SingleAnimation {
                        animation_interval: 0.3,
                        first_index: 1,
                        last_index: 1,
                    },
                    running: SingleAnimation {
                        animation_interval: 0.25,
                        first_index: 0,
                        last_index: 3,
                    },
                },
            },
            grass: AnimationSet {
                sprite_sheet: String::from("spritesheet.png"),
                sprite_sheet_handle: None,
                atlas_tile_columns: 4,
                atlas_tile_rows: 4,
                texture_size: Vec2::new(16.0, 16.0),
                scale: Vec2::new(2.0, 2.0),
                snappy_animations: false,
                animation_class: AnimationClass::Simple {
                    simple: SingleAnimation {
                        animation_interval: 0.7,
                        first_index: 8,
                        last_index: 8,
                    },
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
            ConfigurationSetId::Grass => {
                &self.grass
            }
        }
    }
}

#[derive(Reflect, Default, Resource, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
pub struct AnimationConfiguration {
    pub player: AnimationSet,
    pub sheep: AnimationSet,
    pub grass: AnimationSet,
}

#[derive(Reflect, Default, Resource, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
pub struct AnimationSet {
    #[reflect(ignore)]
    sprite_sheet: String,
    #[reflect(ignore)]
    sprite_sheet_handle: Option<Handle<TextureAtlas>>,
    #[reflect(ignore)]
    atlas_tile_columns: usize,
    #[reflect(ignore)]
    atlas_tile_rows: usize,
    #[reflect(ignore)]
    texture_size: Vec2,
    #[reflect(ignore)]
    scale: Vec2,
    snappy_animations: bool,
    // Should we change animations frame-perfectly or wait until the next?
    animation_class: AnimationClass,
}

impl AnimationSet {
    fn get_anim(self: &Self, anim: AnimationType) -> &SingleAnimation {
        match &self.animation_class {
            AnimationClass::Simple { simple } => { &simple }
            AnimationClass::Actor { idle, running, .. } => {
                match anim {
                    AnimationType::Idle => &idle,
                    AnimationType::Running => &running,
                }
            }
        }
    }
}

#[derive(Reflect, Resource, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
enum AnimationClass {
    Simple {
        simple: SingleAnimation,
    },
    Actor {
        idle: SingleAnimation,
        running: SingleAnimation,
        run_threshold_fraction: f32,
        flip_threshold_fraction: f32,
    },
}

impl Default for AnimationClass {
    fn default() -> Self {
        AnimationClass::Simple { simple: SingleAnimation::default() }
    }
}

#[derive(Reflect, Default, Resource, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
struct SingleAnimation {
    animation_interval: f32,
    first_index: usize,
    last_index: usize,
}

impl FromReflect for SingleAnimation {
    fn from_reflect(_reflect: &dyn Reflect) -> Option<Self> {
        None
    }
}

#[derive(Component, Default)]
struct AnimationStates {
    current: AnimationType,
    next: AnimationType,
    next_flip: bool,
}

#[derive(Copy, Clone, PartialEq, Default)]
enum AnimationType {
    #[default]
    Idle,
    Running,
}

pub fn load_sprite_sheets(asset_server: Res<AssetServer>, texture_atlases: &mut ResMut<Assets<TextureAtlas>>, anim_config: &mut AnimationConfiguration) {
    macro_rules! load {
        ($name:ident, $anim:ident) => {
            let texture_handle = asset_server.load(&anim_config.$name.sprite_sheet);
            let texture_atlas = TextureAtlas::from_grid(
                texture_handle,
                anim_config.$name.texture_size,
                anim_config.$name.atlas_tile_columns,
                anim_config.$name.atlas_tile_rows,
                None,
                None,
            );
            let texture_atlas_handle = texture_atlases.add(texture_atlas);
            anim_config.$name.sprite_sheet_handle = Some(texture_atlas_handle);
        };
    }
    load!(player, idle);
    load!(player, running);
    load!(sheep, idle);
    load!(sheep, running);
    load!(grass, simple);
}


#[derive(Bundle)]
pub struct AnimationBundle {
    sprite_sheet: SpriteSheetBundle,
    animation_timer: AnimationTimer,
    indices: AnimationIndices,
    states: AnimationStates,
}

impl AnimationBundle {
    pub fn from(config_set: &AnimationSet, position: Vec3) -> Self {
        let default_animation = match &config_set.animation_class {
            AnimationClass::Simple { simple } => { simple }
            AnimationClass::Actor { idle, .. } => { idle }
        };
        Self {
            sprite_sheet: SpriteSheetBundle {
                texture_atlas: config_set.sprite_sheet_handle.clone().expect(
                    &format!("all sprite sheets should be loaded for the game to run, missing {}", config_set.sprite_sheet)),
                sprite: TextureAtlasSprite {
                    index: default_animation.first_index,
                    custom_size: Some(config_set.texture_size * config_set.scale),
                    ..default()
                },
                transform: Transform::from_translation(position),
                ..default()
            },
            animation_timer: AnimationTimer(Timer::from_seconds(default_animation.animation_interval, TimerMode::Repeating)),
            indices: AnimationIndices { first: default_animation.first_index, last: default_animation.last_index },
            states: AnimationStates::default(),
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
        &mut AnimationStates,
        &mut AnimationIndices,
        &ConfigurationSetId,
        Option<&Velocity>,
    )>,
    config: Res<Configuration>,
) {
    for (
        mut sprite,
        mut timer,
        mut states,
        mut indices,
        config_id,
        velocity
    ) in query.iter_mut() {
        let config_set = config.animation.get_set(config_id);
        if let Some(velocity) = velocity {
            match &config_set.animation_class {
                AnimationClass::Actor { run_threshold_fraction, flip_threshold_fraction, .. } => {
                    let max_speed = config.get_set(config_id).max_speed;
                    let speed_fraction = velocity.linvel.length() / max_speed;
                    if speed_fraction > *run_threshold_fraction {
                        states.next = AnimationType::Running;
                    } else {
                        states.next = AnimationType::Idle;
                    }
                    let x_speed_fraction = velocity.linvel.x / max_speed;
                    if x_speed_fraction > *flip_threshold_fraction {
                        states.next_flip = true;
                    } else if x_speed_fraction < -*flip_threshold_fraction {
                        states.next_flip = false;
                    }
                }
                _ => {}
            }
        }
        if timer.just_finished() || config_set.snappy_animations {
            if states.current != states.next {
                let animation = config_set.get_anim(states.next);
                let animation_interval = if animation.animation_interval == 0.0 { 0.3 } else { animation.animation_interval };
                timer.set_elapsed(Duration::from_secs_f32(0.0));
                timer.set_duration(Duration::from_secs_f32(animation_interval));
                indices.first = animation.first_index;
                indices.last = animation.last_index;
                sprite.index = indices.first;
                states.current = states.next;
            }
            if sprite.flip_x != states.next_flip {
                sprite.flip_x = states.next_flip
            }
        }
        timer.tick(Duration::from_secs_f32(TIME_STEP));
        if timer.just_finished() {
            sprite.index = if sprite.index == indices.last {
                indices.first
            } else {
                sprite.index + 1
            }
        }
    }
}
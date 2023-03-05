use crate::imports::*;

#[derive(Default)]
pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_startup_system(load_sprite_sheets)
            .add_system_set(
                SystemSet::new()
                    .label("animation")
                    .after("motion")
                    // .with_run_criteria(FixedTimestep::step(TIME_STEP as f64))
                    .with_system(animate_sprite)
            );
    }
}


#[derive(Component, Default)]
pub struct AnimationStates {
    current: AnimationType,
    next: AnimationType,
    next_flip: bool,
}

#[derive(Component, Default)]
pub struct RandomInitAnimation;

#[derive(Copy, Clone, PartialEq, Default)]
pub enum AnimationType {
    #[default]
    Idle,
    Running,
}


#[derive(Bundle)]
pub struct AnimationBundle {
    pub sprite_sheet: SpriteSheetBundle,
    pub animation_timer: AnimationTimer,
    pub states: AnimationStates,
}

impl AnimationBundle {
    pub fn from(anim_sheet: &AnimationSheet, position: Vec3) -> Self {
        let default_animation = match &anim_sheet.animation_class {
            AnimationClass::Simple { simple } => { simple }
            AnimationClass::Actor { idle, .. } => { idle }
        };
        Self {
            sprite_sheet: SpriteSheetBundle {
                texture_atlas: anim_sheet.sprite_sheet_handle.clone().expect(
                    &format!("all sprite sheets should be loaded for the game to run, missing {}", anim_sheet.sprite_sheet)),
                sprite: TextureAtlasSprite {
                    index: anim_sheet.clamp_index(default_animation.first_index),
                    custom_size: Some(anim_sheet.texture_size * GLOBAL_TEXTURE_SCALE),
                    ..default()
                },
                transform: Transform::from_translation(position),
                ..default()
            },
            animation_timer: AnimationTimer(Timer::from_seconds(default_animation.animation_interval, TimerMode::Repeating)),
            states: AnimationStates::default(),
        }
    }
}
impl AnimationSheet {
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
    fn get_num_textures(self: &Self) -> usize {
        self.atlas_tile_columns * self.atlas_tile_rows
    }
    fn clamp_index(self: &Self, index: usize) -> usize {
        std::cmp::min(self.get_num_textures() - 1, index)
    }
}
#[derive(Component, Deref, DerefMut)]
pub struct AnimationTimer(pub Timer);

pub fn animate_sprite(
    mut query: Query<(
        &mut TextureAtlasSprite,
        &mut AnimationTimer,
        &mut AnimationStates,
        &ConfigurationSetId,
        Option<&Velocity>,
    )>,
    config: Res<Configuration>,
    time: Res<Time>,
) {
    for (
        mut sprite,
        mut timer,
        mut states,
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
        let set = config.animation.get_set(config_id);
        let next_anim_config = set.get_anim(states.next);
        if timer.just_finished() || config_set.snappy_animations {
            if states.current != states.next {
                let animation = config_set.get_anim(states.next);
                let animation_interval = if animation.animation_interval == 0.0 { 0.3 } else { animation.animation_interval };
                timer.set_elapsed(Duration::from_secs_f32(0.0));
                timer.set_duration(Duration::try_from_secs_f32(animation_interval).unwrap_or(Duration::from_secs(1)));
                sprite.index = set.clamp_index(next_anim_config.first_index);
                states.current = states.next;
            }
            if sprite.flip_x != states.next_flip {
                sprite.flip_x = states.next_flip
            }
        }
        let current_anim_config = set.get_anim(states.current);
        timer.tick(time.delta());
        if timer.just_finished() {
            let sprite_index = if sprite.index == current_anim_config.last_index {
                current_anim_config.first_index
            } else {
                sprite.index + 1
            };
            sprite.index = set.clamp_index(sprite_index)
        }
    }
}
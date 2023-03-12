use crate::imports::*;

pub const GLOBAL_TEXTURE_SCALE: f32 = 2.0;

impl Configuration {
    pub fn new() -> Self {
        Self {
            zoom: 1.0,
            animation: AnimationConfiguration::new(),
            player: ConfigurationSet {
                max_speed: 300.0,
            },
            sheep: ConfigurationSet {
                max_speed: 100.0,
            },
            flocking: FlockingConfiguration {
                alignment_enabled: true,
                alignment_distance: 60.0,
                alignment_scale: 100.0,
                alignment_distance_cap_fraction: 0.6,
                cohesion_enabled: true,
                cohesion_velocity_scale: true,
                cohesion_distance: 300.0,
                cohesion_scale: 1.5,
                separation_enabled: true,
                separation_distance: 30.0,
                separation_scale: 30.0,
            },
            runner: RunnerConfiguration {
                scale: 10.0,
                speed_fraction: 1.4,
                scare_distance: 160.0,
            },
            grazing_scale: 1.0,
            inertia_scale: 10.0,
            debug_lines: DebugLineConfiguration {
                enable: false,
                // red: DebugLineType::None,
                // green: DebugLineType::None,
                // blue: DebugLineType::None,
                // gray: DebugLineType::None,
                red: DebugLineType::AlignmentInfluence,
                green: DebugLineType::CohesionInfluence,
                blue: DebugLineType::InertiaInfluence,
                gray: DebugLineType::TotalInfluence,
            },
        }
    }
    pub fn get_set<'a>(self: &'a Self, id: &ConfigurationSetId) -> &'a ConfigurationSet {
        match id {
            ConfigurationSetId::Player => {
                &self.player
            }
            ConfigurationSetId::Sheep => {
                &self.sheep
            }
            _ => {
                &self.sheep
            }
        }
    }
}

#[derive(Reflect, Default, Resource, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
pub struct Configuration {
    pub zoom: f32,
    pub animation: AnimationConfiguration,
    pub player: ConfigurationSet,
    pub sheep: ConfigurationSet,
    pub flocking: FlockingConfiguration,
    pub runner: RunnerConfiguration,
    pub grazing_scale: f32,
    pub inertia_scale: f32,
    pub debug_lines: DebugLineConfiguration,
}

impl AnimationConfiguration {
    pub fn new() -> Self {
        Self {
            player: AnimationSheet {
                sprite_sheet: String::from("collie_sheet.png"),
                sprite_sheet_handle: None,
                atlas_tile_columns: 4,
                atlas_tile_rows: 4,
                texture_size: Vec2::new(20.0, 16.0),
                snappy_animations: true,
                animation_class: AnimationClass::Actor {
                    run_threshold_fraction: 0.3,
                    flip_threshold_fraction: 0.1,
                    idle: SingleAnimation {
                        animation_interval: 0.3,
                        first_index: 0,
                        last_index: 2,
                    },
                    running: SingleAnimation {
                        animation_interval: 0.15,
                        first_index: 4,
                        last_index: 5,
                    },
                },
            },

            sheep: AnimationSheet {
                sprite_sheet: String::from("sheep_sheet.png"),
                sprite_sheet_handle: None,
                atlas_tile_columns: 6,
                atlas_tile_rows: 3,
                texture_size: Vec2::new(16.0, 16.0),
                snappy_animations: false,
                animation_class: AnimationClass::Actor {
                    run_threshold_fraction: 0.3,
                    flip_threshold_fraction: 0.2,
                    idle: SingleAnimation {
                        animation_interval: 0.3,
                        first_index: 0,
                        last_index: 2,
                    },
                    running: SingleAnimation {
                        animation_interval: 0.15,
                        first_index: 7,
                        last_index: 10,
                    },
                },
            },
            fence_horizontal: AnimationSheet {
                sprite_sheet: String::from("fence_horizontal.png"),
                sprite_sheet_handle: None,
                atlas_tile_columns: 1,
                atlas_tile_rows: 1,
                texture_size: Vec2::new(19.0, 8.0),
                snappy_animations: false,
                animation_class: AnimationClass::Simple {
                    simple: SingleAnimation {
                        animation_interval: 1.0,
                        first_index: 0,
                        last_index: 0,
                    },
                },
            },
            fence_vertical: AnimationSheet {
                sprite_sheet: String::from("fence_vertical.png"),
                sprite_sheet_handle: None,
                atlas_tile_columns: 1,
                atlas_tile_rows: 1,
                texture_size: Vec2::new(2.0, 26.0),
                snappy_animations: false,
                animation_class: AnimationClass::Simple {
                    simple: SingleAnimation {
                        animation_interval: 1.0,
                        first_index: 0,
                        last_index: 0,
                    },
                },
            },
            grass: AnimationSheet {
                sprite_sheet: String::from("spritesheet.png"),
                sprite_sheet_handle: None,
                atlas_tile_columns: 4,
                atlas_tile_rows: 4,
                texture_size: Vec2::new(16.0, 16.0),
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
    pub fn get_set(self: &Self, id: &ConfigurationSetId) -> &AnimationSheet {
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
            ConfigurationSetId::FenceHorizontal => {
                &self.fence_horizontal
            }
            ConfigurationSetId::FenceVertical => {
                &self.fence_vertical
            }
        }
    }
}
pub fn load_sprite_sheets(asset_server: Res<AssetServer>, mut texture_atlases: ResMut<Assets<TextureAtlas>>, mut config: ResMut<Configuration>) {
    println!("Loading sprite sheets");
    let mut anim_config = &mut config.animation;
    macro_rules! load {
        ($name:ident, $anim:ident) => {
            let texture_handle = asset_server.load(&anim_config.$name.sprite_sheet);
            let mut texture_atlas = TextureAtlas::from_grid(
                texture_handle,
                anim_config.$name.texture_size,
                anim_config.$name.atlas_tile_columns,
                anim_config.$name.atlas_tile_rows,
                None,
                None,
            );
            for mut rect in texture_atlas.textures.iter_mut() {
                rect.min = rect.min - 0.5;
                rect.max = rect.max - 0.5;
            }
            let texture_atlas_handle = texture_atlases.add(texture_atlas);
            anim_config.$name.sprite_sheet_handle = Some(texture_atlas_handle);
        };
    }
    load!(player, idle);
    load!(player, running);
    load!(sheep, idle);
    load!(sheep, running);
    load!(fence_horizontal, simple);
    load!(fence_vertical, simple);
    load!(grass, simple);
}


#[derive(Reflect, Default, Resource, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
pub struct AnimationConfiguration {
    pub player: AnimationSheet,
    pub sheep: AnimationSheet,
    pub fence_horizontal: AnimationSheet,
    pub fence_vertical: AnimationSheet,
    pub grass: AnimationSheet,
}

#[derive(Reflect, Default, Resource, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
pub struct AnimationSheet {
    #[reflect(ignore)]
    pub sprite_sheet: String,
    #[reflect(ignore)]
    pub sprite_sheet_handle: Option<Handle<TextureAtlas>>,
    #[reflect(ignore)]
    pub atlas_tile_columns: usize,
    #[reflect(ignore)]
    pub atlas_tile_rows: usize,
    #[reflect(ignore)]
    pub texture_size: Vec2,
    pub snappy_animations: bool,
    // Should we change animations frame-perfectly or wait until the next?
    pub animation_class: AnimationClass,
}


#[derive(Reflect, Resource, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
pub enum AnimationClass {
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
pub struct SingleAnimation {
    pub animation_interval: f32,
    pub first_index: usize,
    pub last_index: usize,
}

impl FromReflect for SingleAnimation {
    fn from_reflect(_reflect: &dyn Reflect) -> Option<Self> {
        None
    }
}


#[derive(Component)]
pub enum ConfigurationSetId {
    Player,
    Sheep,
    FenceHorizontal,
    FenceVertical,
    Grass,
}

#[derive(Reflect, Default, InspectorOptions)]
#[reflect(InspectorOptions)]
pub struct ConfigurationSet {
    #[inspector(min = 0.0)]
    pub max_speed: f32,
}


#[derive(Reflect, Default, Resource, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
pub struct FlockingConfiguration {
    pub alignment_enabled: bool,
    #[inspector(min = 0.0)]
    pub alignment_distance: f32,
    #[inspector(min = 0.0)]
    pub alignment_scale: f32,
    #[inspector(min = 0.0)]
    pub alignment_distance_cap_fraction: f32,
    pub cohesion_enabled: bool,
    pub cohesion_velocity_scale: bool,
    #[inspector(min = 0.0)]
    pub cohesion_distance: f32,
    #[inspector(min = 0.0)]
    pub cohesion_scale: f32,
    pub separation_enabled: bool,
    #[inspector(min = 0.0)]
    pub separation_distance: f32,
    #[inspector(min = 0.0)]
    pub separation_scale: f32,
}

#[derive(Reflect, Default, InspectorOptions)]
#[reflect(InspectorOptions)]
pub struct RunnerConfiguration {
    #[inspector(min = 0.0)]
    pub scale: f32,
    #[inspector(min = 0.0)]
    pub speed_fraction: f32,
    #[inspector(min = 0.0)]
    pub scare_distance: f32,
}

#[derive(Reflect, Default)]
pub enum DebugLineType {
    #[default]
    None,
    AlignmentInfluence,
    CohesionInfluence,
    SeparationInfluence,
    RunnerInfluence,
    RunnerUnmodifiedInfluence,
    RunnerMaxInfluence,
    GrazingInfluence,
    InertiaInfluence,
    TotalInfluence,
    MaxInfluence,
}

#[derive(Reflect, Default)]
pub struct DebugLineConfiguration {
    pub enable: bool,
    pub red: DebugLineType,
    pub green: DebugLineType,
    pub blue: DebugLineType,
    pub gray: DebugLineType,
}


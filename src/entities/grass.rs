use crate::imports::*;

#[derive(Default)]
pub struct GrassPlugin;

impl Plugin for GrassPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_grass);
    }
}

pub fn spawn_grass(
    mut commands: Commands,
    config: Res<Configuration>,
) {
    let distribution = Uniform::new(-1000.0, 1000.0);
    for _ in 0..300 {
        let x = distribution.sample(&mut rand::thread_rng());
        let y = distribution.sample(&mut rand::thread_rng());
        commands.spawn(GrassBundle::new(&config.animation.grass, Vec3::new(x, y, 0.0)));
    }
}

#[derive(Bundle)]
pub struct GrassBundle {
    animation_bundle: AnimationBundle,
    name: Name,
    config_set_id: ConfigurationSetId,
}

impl GrassBundle {
    pub fn new(config_set: &AnimationSheet, position: Vec3) -> Self {
        GrassBundle {
            animation_bundle: AnimationBundle::from(config_set, position),
            name: Name::new("Grass"),
            config_set_id: ConfigurationSetId::Grass,
        }
    }
}

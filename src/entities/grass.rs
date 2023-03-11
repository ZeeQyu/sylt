use crate::imports::*;

#[derive(Default)]
pub struct GrassPlugin;

impl Plugin for GrassPlugin {
    fn build(&self, app: &mut App) {
        // app.add_enter_system(GameState::Game, spawn_grass);
        // app.add_exit_system(GameState::Game, despawn_grass);
    }
}

const GRASS_NAME: &str = "Grass";

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
pub fn despawn_grass(
    mut commands: Commands,
    query: Query<Entity, With<IsGrass>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

#[derive(Bundle)]
pub struct GrassBundle {
    animation_bundle: AnimationBundle,
    name: Name,
    config_set_id: ConfigurationSetId,
    is_grass: IsGrass,
}

impl GrassBundle {
    pub fn new(config_set: &AnimationSheet, position: Vec3) -> Self {
        GrassBundle {
            animation_bundle: AnimationBundle::from(config_set, position),
            name: Name::new(GRASS_NAME),
            config_set_id: ConfigurationSetId::Grass,
            is_grass: IsGrass,
        }
    }
}

#[derive(Component)]
pub struct IsGrass;
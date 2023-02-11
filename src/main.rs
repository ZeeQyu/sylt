use bevy::{
    prelude::*,
    time::FixedTimestep,
    // sprite::collide_aabb::{collide, Collision},
};

const TIME_STEP: f32 = 1.0 / 60.0;

fn main() {
    let background_color = Color::rgb_u8(46 as u8, 34 as u8, 47 as u8);
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(background_color))
        .add_startup_system(setup)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(TIME_STEP as f64))
                .with_system(move_player)
        )
        .run();
}

fn setup(
    mut commands: Commands,
    // mut meshes: ResMut<Assets<Mesh>>,
    // mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(Camera2dBundle::default());

    spawn_player(&mut commands, &asset_server);
    spawn_sheep(&mut commands, &asset_server);
}


#[derive(Component)]
struct Collider;

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Sheep;

#[derive(Bundle)]
struct SheepBundle {
    sprite_bundle: SpriteBundle,
    collider: Collider,
    sheep: Sheep,
}

impl SheepBundle {
    fn new(position: Vec2, image: Handle<Image>) -> SheepBundle {
        SheepBundle {
            sprite_bundle: SpriteBundle {
                // transform: Transform {
                //     translation: Vec3::new(position.x, position.y, 0.0),
                //     scale: Vec3::new(20.0, 20.0, 20.0),
                //     ..default()
                // },
                transform: Transform {
                    translation: Vec3::new(position.x, position.y, 0.0),
                    scale: Vec3::splat(4.0),
                    ..default()
                },
                // sprite: Sprite {
                //     color: Color::rgb(0.9, 0.3, 0.3),
                //     ..default()
                // },
                texture: image,
                ..default()
            },
            collider: Collider,
            sheep: Sheep,
        }
    }
}

fn spawn_player(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    let player_texture: Handle<Image> = asset_server.load("Collie.png");
    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: Vec3::new(-400.0, 0.0, 0.0),
                scale: Vec3::splat(4.0),
                ..default()
            },
            texture: player_texture,
            ..default()
        },
        Player
    ));
}


fn spawn_sheep(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    let sheep_texture: Handle<Image> = asset_server.load("Sheep.png");
    for x_counter in -3..3 {
        for y_counter in -3..3 {
            let spacing: f32 = 50.0;
            let position = Vec2::new(
                (x_counter as f32) * &spacing,
                (y_counter as f32) * &spacing,
            );
            commands.spawn(SheepBundle::new(position, sheep_texture.clone()));
        }
    }
}

fn move_player(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<Player>>,
) {
    let mut player_transform = query.single_mut();
    let mut direction = Vec2::new(0.0, 0.0);
    if keyboard_input.pressed(KeyCode::Left) {
        direction.x -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::Right) {
        direction.x += 1.0;
    }
    if keyboard_input.pressed(KeyCode::Up) {
        direction.y += 1.0;
    }
    if keyboard_input.pressed(KeyCode::Down) {
        direction.y -= 1.0;
    }
    let player_speed: f32 = 200.0;
    let player_position = Vec3::new(
        player_transform.translation.x + direction.x * player_speed * TIME_STEP,
        player_transform.translation.y + direction.y * player_speed * TIME_STEP,
        0.0,
    );
    player_transform.translation = player_position;
}
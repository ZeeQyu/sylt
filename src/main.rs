use bevy::{
    prelude::*,
    time::FixedTimestep,
    // sprite::collide_aabb::{collide, Collision},
};

const TIME_STEP: f32 = 1.0 / 60.0;

fn main() {
    let background_color = Color::rgb_u8(46 as u8, 34 as u8, 47 as u8);
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
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
    // mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    // let texture_handle = asset_server.load("spritesheet.png");
    // let texture_atlas_handle = texture_atlases.add(texture_atlas);
    // let texture_atlas = texture_atlases.get(texture_atlas_handle).unwrap();
    // let player_sprite =
    commands.spawn(Camera2dBundle::default());
    let player_texture: Handle<Image> = asset_server.load("Collie.png");
    let sheep_texture: Handle<Image> = asset_server.load("Sheep.png");
    spawn_player(&mut commands, player_texture);
    spawn_sheep(&mut commands, sheep_texture);
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
                transform: Transform {
                    translation: Vec3::new(position.x, position.y, 0.0),
                    scale: Vec3::splat(2.0),
                    ..default()
                },
                texture: image,
                ..default()
            },
            collider: Collider,
            sheep: Sheep,
        }
    }
}

fn spawn_player(commands: &mut Commands, player_texture: Handle<Image>) {
    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: Vec3::new(-400.0, 0.0, 0.0),
                scale: Vec3::splat(2.0),
                ..default()
            },
            texture: player_texture,
            ..default()
        },
        Player
    ));
}


fn spawn_sheep(commands: &mut Commands, sheep_texture: Handle<Image>) {
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
    if keyboard_input.pressed(KeyCode::Left) ||
        keyboard_input.pressed(KeyCode::A) {
        direction.x -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::Right) ||
        keyboard_input.pressed(KeyCode::D){
        direction.x += 1.0;
    }
    if keyboard_input.pressed(KeyCode::Up) ||
        keyboard_input.pressed(KeyCode::W){
        direction.y += 1.0;
    }
    if keyboard_input.pressed(KeyCode::Down) ||
        keyboard_input.pressed(KeyCode::S){
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
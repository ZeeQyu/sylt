use bevy::{
    prelude::*,
    time::FixedTimestep,
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
                .with_system(apply_velocity)
                .with_system(apply_player_input)
                .with_system(move_sheep)
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
    for x_counter in -3..3 {
        for y_counter in -3..3 {
            let spacing: f32 = 50.0;
            let position = Vec2::new(
                (x_counter as f32) * &spacing,
                (y_counter as f32) * &spacing,
            );
            spawn_sheep(&mut commands, &position, sheep_texture.clone());
        }
    }
}


#[derive(Component)]
struct Collider;

#[derive(Component)]
struct Velocity {
    velocity: Vec3,
}
impl Velocity {
    fn new() -> Self {
        Self {
            velocity: Vec3::ZERO
        }
    }
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Sheep;

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
        Player,
        Velocity::new()
    ));
}

fn spawn_sheep(commands: &mut Commands, position: &Vec2, image: Handle<Image>) {
    commands.spawn(
        (
            SpriteBundle {
                transform: Transform {
                    translation: Vec3::new(position.x, position.y, 0.0),
                    scale: Vec3::splat(
                        2.0),
                    ..default()
                },
                texture: image,
                ..default()
            },
            Collider,
            Sheep,
            Velocity::new(),
        )
    );
}

fn apply_player_input(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Velocity, With<Player>>,
) {
    let mut direction = Vec2::new(0.0, 0.0);
    if keyboard_input.pressed(KeyCode::Left) ||
        keyboard_input.pressed(KeyCode::A) {
        direction.x -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::Right) ||
        keyboard_input.pressed(KeyCode::D) {
        direction.x += 1.0;
    }
    if keyboard_input.pressed(KeyCode::Up) ||
        keyboard_input.pressed(KeyCode::W) {
        direction.y += 1.0;
    }
    if keyboard_input.pressed(KeyCode::Down) ||
        keyboard_input.pressed(KeyCode::S) {
        direction.y -= 1.0;
    }
    let player_speed: f32 = 200.0;
    let mut player_velocity = query.single_mut();
    direction = direction.normalize_or_zero();
    player_velocity.velocity = Vec3::new(
        direction.x * player_speed,
        direction.y * player_speed,
        0.0,
    );
}

fn move_sheep(
    mut player_query: Query<(&Transform, With<Player>)>,
    mut sheep_query: Query<((&mut Velocity, &Transform), With<Sheep>)>,
) {
    let (Transform { translation: player_position, .. }, ()) = player_query.single();
    for (
        (
            mut velocity,
            &Transform { translation: sheep_position, .. }
        ),
        ()
    ) in sheep_query.iter_mut() {
        let scare_distance = 200.0;
        let sheep_speed = 120.0;
        if sheep_position.distance(*player_position) < scare_distance {
            velocity.velocity = (*player_position - sheep_position).normalize_or_zero() * sheep_speed;
        }
    }
}

fn apply_velocity(
    mut query: Query<(&mut Transform, &Velocity)>
) {
    for (mut transform, Velocity { velocity }) in query.iter_mut() {
        transform.translation += *velocity * TIME_STEP;
    }
}
use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;

use bevy::math::bounding::{Aabb2d, BoundingCircle, BoundingVolume, IntersectsVolume};
use bevy::window::WindowResolution;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            // Wasm builds will check for meta files (that don't exist) if this isn't set.
            // This causes errors and even panics in web builds on itch.
            // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
            meta_check: AssetMetaCheck::Never,
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, (player_movement))
        .add_systems(Update, (apply_gravity_and_drag, apply_velocity).chain())
        .add_systems(Update, (move_camera, read_cursor_world_pos))
        .run();
}

fn setup(
    mut commands: Commands, 
    asset_server: Res<AssetServer>,
    mut window: Single<&mut Window>,
) 
{
    window.resolution = WindowResolution::new(1280.0, 720.0);

    commands.insert_resource(CursorTimer(Timer::from_seconds(1.0, TimerMode::Repeating)));

    commands.spawn((Camera2d, MyCamera, Transform::default()));
    // commands.spawn(Sprite {
    //     image: asset_server.load("ducky.png"),
    //     ..Default::default()
    // });

    let mut platform = Transform::from_xyz(0.0, -100.0, 0.0);
    platform.scale = Vec3::new(200.0, 20.0, 1.0);

    commands.spawn((
        platform,
        Platform,
        Sprite::from_color(Color::Srgba(Srgba::RED), [1.0, 1.0].into())
    ));
    commands.spawn((
        Player,
        Transform::default(),
        Sprite::from_color(Color::Srgba(Srgba::rgb_u8(36, 240, 36)), [4.0, 4.0].into()),
        ExperiencesGravity,
        Velocity::default(),
    ));
}

#[derive(Component, Copy, Clone, Debug, Default)]
struct Player;

#[derive(Component, Copy, Clone, Debug, Default)]
struct Platform;

#[derive(Component, Copy, Clone, Debug, Default)]
struct MyCamera;

#[derive(Resource)]
struct CursorTimer(Timer);

#[derive(Component, Copy, Clone, Debug, Default)]
#[require(Velocity)]
struct ExperiencesGravity;

#[derive(Component, Copy, Clone, Debug, Default)]
struct Velocity(Vec2);

fn player_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &mut Velocity), With<Player>>,
) 
{
    for (mut transform, mut velocity) in &mut query 
    {
        if keyboard_input.pressed(KeyCode::KeyA) {
            // transform.translation.x -= 1.0;
            velocity.0.x -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            // transform.translation.x += 1.0;
            velocity.0.x += 1.0;
        }

        if keyboard_input.just_pressed(KeyCode::Space)
        {
            // Apply an upward force to the player
            velocity.0.y += 10.0; // Jump force
        }
        // if keyboard_input.pressed(KeyCode::KeyW) {
        //     transform.translation.y += 1.0;
        // }
        // if keyboard_input.pressed(KeyCode::KeyS) {
        //     transform.translation.y -= 1.0;
        // }
    }
}

fn apply_gravity_and_drag(
    query: Query<&mut Velocity, With<ExperiencesGravity>>,
    time: Res<Time>,
)
{
    for mut obj in query
    {
        // Apply gravity to the velocity
        obj.0.y -= 9.81 * time.delta_secs(); // Gravity acceleration

        obj.0.x *= 0.99; // Apply drag to the x velocity
    }
}

fn apply_velocity(
    query: Query<(&mut Transform, &Velocity)>,
    time: Res<Time>,
)
{
    for (mut obj, vel) in query
    {
        obj.translation += Vec3::new(vel.0.x, vel.0.y, 0.0) * time.delta_secs();
    }
}

fn check_collisions(
    mut commands: Commands,
    player: Single<(Entity, &mut Transform, &mut Velocity), With<Player>>,
    platforms: Query<(Entity, &Transform), With<Platform>>,
)
{
    let (ball_ent, mut ball_transform, mut ball_velocity) = player.into_inner();

    let player_collider = Aabb2d::new(ball_transform.translation.xy(), ball_transform.scale.xy() / 2.0);
    let mut collided = false;

    
}

fn move_camera(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<MyCamera>>,
)
{
    for mut transform in &mut query
    {
        if keyboard_input.pressed(KeyCode::ArrowLeft) {
            transform.translation.x -= 5.0;
        }
        if keyboard_input.pressed(KeyCode::ArrowRight) {
            transform.translation.x += 5.0;
        }
        if keyboard_input.pressed(KeyCode::ArrowUp) {
            transform.translation.y += 5.0;
        }
        if keyboard_input.pressed(KeyCode::ArrowDown) {
            transform.translation.y -= 5.0;
        }
    }
}

fn read_cursor_world_pos(
    window: Single<&Window>,
    camera: Single<&Projection, With<Camera2d>>,
    mut cursor_timer: ResMut<CursorTimer>,
    time: Res<Time>,
)
{
    let cursor_pos = window.cursor_position();

    let a = window.physical_cursor_position();

    
    if cursor_timer.0.tick(time.delta()).just_finished()
    {
        println!("Cursor position: {:?}", cursor_pos);
    
        println!("Camera projection: {:?}", camera.into_inner());

    }

    // if let Some(world_pos) = cursor_pos.and_then(|cursor| camera.) {
    //     println!("Cursor world position: {:?}", world_pos);
    // } else {
    //     println!("Cursor is not over the window.");
    // }
}
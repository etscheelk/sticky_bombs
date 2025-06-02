use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(20.0))
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_systems(Startup, setup_graphics)
        .add_systems(Startup, setup_physics)
        .add_systems(Update, (print_ball_altitude, ball_jump))
        .run();
}

fn setup_graphics(mut commands: Commands) {
    // Add a camera so we can see the debug-render.
    commands.spawn(Camera2d::default());
}

fn setup_physics(mut commands: Commands) {
    /* Create the ground. */
    // commands
    //     .spawn(Collider::cuboid(500.0, 50.0))
    //     .insert(Transform::from_xyz(0.0, -100.0, 0.0));

    commands.spawn((
        Collider::cuboid(500.0, 50.0),
        Transform::from_xyz(0.0, -100.0, 0.0),
        RigidBody::Fixed,
        Friction::coefficient(0.5),
        Restitution::coefficient(0.5),
    ));

    /* Create the bouncing ball. */
    // commands
    //     .spawn(RigidBody::Dynamic)
    //     .insert(Collider::ball(50.0))
    //     .insert(Restitution::coefficient(0.7))
    //     .insert(Transform::from_xyz(0.0, 400.0, 0.0));

    commands.spawn((
        Ball,
        RigidBody::Dynamic,
        Velocity::default(),
        Collider::ball(50.0),
        Restitution::coefficient(0.7),
        Transform::from_xyz(0.0, 400.0, 0.0),
        Friction::coefficient(0.5),
        Damping {
            linear_damping: 0.9,
            angular_damping: 0.9,
        },
        
    ));
}

#[derive(Component)]
struct Ball;

fn ball_jump(
    mut commands: Commands,
    mut ball: Single<(Entity, &mut Velocity, &Transform), With<Ball>>,
    keyboard: Res<ButtonInput<KeyCode>>,
)
{
    let (ent, mut ball, transform) = ball.into_inner();

    if keyboard.just_pressed(KeyCode::Space)
    {
        ball.linvel.y += 1000.0;
        // commands.entity(ent).insert(ExternalImpulse::at_point([0.0, 100.0].into(), [0.0, 0.0].into(), transform.translation.xy()));
    }

    if keyboard.pressed(KeyCode::KeyA)
    {
        ball.angvel += 0.1;
    }

    if keyboard.pressed(KeyCode::KeyD)
    {
        ball.angvel -= 0.1;
    }
}

fn print_ball_altitude(positions: Query<&Transform, With<Ball>>) {
    for transform in positions.iter() {
        println!("Ball altitude: {}", transform.translation.y);
    }
}
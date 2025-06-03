use bevy::{log::tracing_subscriber::filter::Targets, prelude::*};
use bevy_rapier2d::{prelude::*, rapier::prelude::CollisionEventFlags};

fn main() {
    App::new()
    .add_plugins(DefaultPlugins)
    .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
    .add_plugins(RapierDebugRenderPlugin::default())
    .add_systems(Startup, setup_graphics)
    .add_systems(Startup, setup_physics)
    .add_systems(Update, (print_ball_altitude, ball_jump, player_move))
    .add_systems(Update, (diverge_collision_events, sensor_collision_events).chain())
    .run();
}

fn setup_graphics(mut commands: Commands) {
    // Add a camera so we can see the debug-render.
    commands.spawn(Camera2d::default());
}

fn setup_physics(mut commands: Commands) {
    /* Create the ground. */
    commands.spawn((
        Collider::cuboid(500.0, 50.0),
        Transform::from_xyz(0.0, -100.0, 0.0),
        RigidBody::Fixed,
        Friction::coefficient(0.5),
        Restitution::coefficient(0.5),
    ));

    commands.spawn((
        Collider::cuboid(200.0, 12.0),
        Transform::from_xyz(0.0, -36.0, 0.0),
        RigidBody::Fixed,
        Friction::coefficient(0.5),
        Restitution::coefficient(0.5),
        // Sprite::from_color(Color::Srgba(Srgba::RED), [20.0, 20.0].into()),
    ));

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
    ))
    .with_children(|ent|
    {
        ent.spawn(BombPlaceSpotBundle::ball_with_radius(75.0))
        .with_child((
            Sprite::from_color(Color::Srgba(Srgba::RED), [20.0, 20.0].into()),
            Visibility::Hidden,
            Bomb,
            Transform::default(),
        ));
    });

    commands.spawn((
        Player,
        KinematicCharacterController {
            apply_impulse_to_dynamic_bodies: true,
            autostep: Some(CharacterAutostep {
                include_dynamic_bodies: true,
                max_height: CharacterLength::Relative(0.25),
                min_width: CharacterLength::Relative(0.5),
            }),
            ..Default::default()
        },
        Transform::from_xyz(120.0, 200.0, 0.0),
        Velocity::default(),
        Collider::cuboid(30.0, 60.0),
        // Restitution::coefficient(0.5),
        RigidBody::KinematicVelocityBased,
        Friction::coefficient(0.2),
        Damping {
            linear_damping: 0.2,
            angular_damping: 0.2,
        }
    ))
    .with_children(|ent|
    {
        ent.spawn(BombPlacerBundle::ball_with_radius(72.0));
        // ent.spawn((
        //     Sprite::from_color(Color::Srgba(Srgba::GREEN), [10.0, 10.0].into()),
        //     Visibility::Hidden,
        //     Bomb
        // ));
    });

    commands.insert_resource(Events::<SensorEvent>::default());
}

#[derive(Bundle, Clone, Debug)]
struct SensorBundle
{
    sensor: Sensor,
    collider: Collider,
    active_events: ActiveEvents,
    collision_groups: CollisionGroups,
    transform: Transform,
}

#[derive(Bundle, Clone, Debug)]
struct BombPlaceSpotBundle
{
    bomb_place_spot: BombPlaceSpot,
    sensor_bundle: SensorBundle,
}

impl BombPlaceSpotBundle
{
    const MEMBERSHIPS:  Group = Group::GROUP_32;
    const FILTERS:      Group = Group::GROUP_2;

    fn ball_with_radius(radius: f32) -> Self
    {
        Self {
            bomb_place_spot: BombPlaceSpot,
            sensor_bundle: SensorBundle {
                sensor: Sensor,
                collider: Collider::ball(radius),
                active_events: ActiveEvents::COLLISION_EVENTS,
                collision_groups: CollisionGroups::new(
                    Self::MEMBERSHIPS, 
                    Self::FILTERS,
                ),
                transform: Transform::default(),
            },
        }
    }
}

#[derive(Bundle, Clone, Debug)]
struct BombPlacerBundle
{
    bomb_proximity_placer: BombPromixityPlacer,
    sensor_bundle: SensorBundle,
}

impl BombPlacerBundle
{
    const MEMBERSHIPS:  Group = Group::GROUP_2;
    const FILTERS:      Group = Group::GROUP_32;

    fn ball_with_radius(radius: f32) -> Self
    {
        Self {
            bomb_proximity_placer: BombPromixityPlacer,
            sensor_bundle: SensorBundle {
                sensor: Sensor,
                collider: Collider::ball(radius),
                active_events: ActiveEvents::COLLISION_EVENTS,
                collision_groups: CollisionGroups::new(
                    Self::MEMBERSHIPS, 
                    Self::FILTERS,
                ),
                transform: Transform::default(),
            },
        }
    }
}

#[derive(Component)]
struct Ball;

#[derive(Component, Clone, Copy, Debug)]
struct BombPlaceSpot;

#[derive(Component)]
struct Player;

#[derive(Component, Clone, Copy, Debug)]
struct BombPromixityPlacer;

#[derive(Component, Clone, Copy, Debug)]
struct Bomb;

fn player_move(
    mut players: Query<(&mut KinematicCharacterController, &mut Velocity), With<Player>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    r_context_mut: Single<&mut RapierContextSimulation>,
    r_config: Single<&RapierConfiguration>,
    time: Res<Time>,
)
{
    let a = r_context_mut.into_inner();

    let gravity = r_config.gravity;

    for (mut player, mut vel) in players.iter_mut()
    {
        let mut init_vel = vel.clone();

        let mut translation = Vec2::ZERO;
        if keyboard.pressed(KeyCode::ArrowLeft)
        {
            init_vel.linvel.x -= 10.0;
        }
        if keyboard.pressed(KeyCode::ArrowRight)
        {
            init_vel.linvel.x += 10.0;
        }
        if keyboard.just_pressed(KeyCode::ArrowUp)
        {
            init_vel.linvel.y += 100.0;
        }

        init_vel.linvel += gravity * time.delta_secs();

        translation += init_vel.linvel * time.delta_secs();

        // *vel = init_vel;
        player.translation = Some(translation / time.delta_secs());
    }

    
}

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

fn diverge_collision_events(
    mut collision_events: EventReader<CollisionEvent>,
    mut sensor_events: EventWriter<SensorEvent>,
    mut commands: Commands,
) 
{
    for event in collision_events.read() {
        match *event {
            CollisionEvent::Started(a, b, t) => {
                if t == CollisionEventFlags::SENSOR
                {
                    sensor_events.write(SensorEvent(a, b, SensorInteraction::Entered));
                }
            }
            CollisionEvent::Stopped(a, b, t) => {
                if t == CollisionEventFlags::SENSOR
                {
                    sensor_events.write(SensorEvent(a, b, SensorInteraction::Exited));
                }
            }
        }
    }
}
    
#[derive(Event, Clone, Copy, Debug)]
struct SensorEvent(Entity, Entity, SensorInteraction);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SensorInteraction
{
    Entered,
    Exited,
}

// Sensor events will occur when at least one of the colliders involved in the collision is a sensor.
// 
// We need to split up these events for the variety of use-cases we may have for sensors.
// a. Bomb placement, by proximity to the player, typically
fn sensor_collision_events(
    mut sensor_events: EventReader<SensorEvent>,
    mut _commands: Commands,
    bomb_placers: Query<Entity, With<BombPromixityPlacer>>,
    // placer_imgs: Query<(Entity, &Children), With<BombPromixityPlacer>>,
    mut bomb_img: Query<(Entity, &ChildOf, &mut Visibility), With<Bomb>>,
    bomb_place_spots: Query<Entity, With<BombPlaceSpot>>,
)
{
    for &SensorEvent(a, b, t) in sensor_events.read()
    {
        // check if one type is able to place a bomb, and the other is a placeable area
        // If successful, do something.
        /*
            Maybe a marker component on the spot that we can highlight?
            Add a reference to the spot on the placer?
            Or, add a reference to the placer on the spot?
         */


        // case 1: a is a placer, b is a spot
        let placer = bomb_placers.get(a);
        let spot = bomb_place_spots.get(b);
        if let (Ok(placer), Ok(spot)) = (placer, spot)
        {
            println!("case 1");
            println!("Placer {:?} is near spot {:?}", placer, spot);
            println!("Sensor interaction: {:?}", t);
        }

        // case 2: a is a spot, b is a placer
        let placer = bomb_placers.get(b);
        let spot = bomb_place_spots.get(a);
        if let (Ok(placer), Ok(spot)) = (placer, spot)
        {
            println!("case 2");
            println!("Placer {:?} is near spot {:?}", placer, spot);
            println!("Sensor interaction: {:?}", t);

            // println!("placer children? {:?}", placer_imgs.get(placer));
            // if let Ok((e,c)) = placer_imgs.get(placer)
            // {
            //     println!("Children of placer: {:?}", c);
            // }

            println!("length of bomb_img query: {:?}", bomb_img.iter().len());

            for (ent, child_of, mut vis) in bomb_img.iter_mut()
            {
                // println!("child_of: {:?}", child_of);
                info!("Child_of: {:?}", child_of);
                // println!("child_of.parent(): {:?}", child_of.parent());

                if child_of.parent() == spot
                {
                    println!("Tried to change visibility");

                    use SensorInteraction::*;
                    match t
                    {
                        Entered => *vis = Visibility::Inherited,
                        Exited => *vis = Visibility::Hidden,
                    };
                }
            }

            // if let Ok((_, mut vis)) = bomb_img.get_mut(spot)
            // {
            //     println!("Tried to change visibility");

            //     use SensorInteraction::*;
            //     match t
            //     {
            //         Entered => *vis = Visibility::Inherited,
            //         Exited => *vis = Visibility::Hidden,
            //     };
            // }
        }

        // case 3: not interested in this event right now
    }
}

fn print_ball_altitude(positions: Query<&Transform, With<Ball>>) 
{
    for transform in positions.iter() {
        // println!("Ball altitude: {}", transform.translation.y);
    }
}
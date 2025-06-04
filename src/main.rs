use bevy::prelude::*;
use bevy::color::palettes::css::GRAY;
use bevy::render::camera::RenderTarget;
use bevy::render::render_resource::{Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages};
use bevy::render::view::RenderLayers;

use bevy::window::WindowResized;
use bevy_rapier2d::rapier::prelude::CollisionEventFlags;
use bevy_rapier2d::prelude::*;

fn main() {
    App::new()
    .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
    .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(8.0))
    .add_plugins(RapierDebugRenderPlugin::default())
    .add_systems(Startup, setup_graphics)
    .add_systems(Startup, setup_physics)
    // .add_systems(FixedUpdate, player_move)
    .add_systems(Update, player_move)
    .add_systems(Update, (print_ball_altitude, ball_jump, fit_canvas))
    .add_systems(Update, (diverge_collision_events, sensor_collision_events).chain())
    .run();
}

const RES_WIDTH: f32 = 640.0;
const RES_HEIGHT: f32 = 360.0;

const PIXEL_PERFECT_LAYERS: RenderLayers = RenderLayers::layer(0);

const HIGH_RES_LAYERS: RenderLayers = RenderLayers::layer(1);

/// Low-resolution texture that contains the pixel-perfect world.
/// Canvas itself is rendered to the high-resolution world.
#[derive(Component)]
struct Canvas;

/// Camera that renders the pixel-perfect world to the [`Canvas`].
#[derive(Component)]
struct InGameCamera;

/// Camera that renders the [`Canvas`] (and other graphics on [`HIGH_RES_LAYERS`]) to the screen.
#[derive(Component)]
struct OuterCamera;


fn setup_graphics(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
) 
{
    // Add a camera so we can see the debug-render.
    // commands.spawn(Camera2d::default());
    let canvas_size = Extent3d {
        width: RES_WIDTH as u32,
        height: RES_HEIGHT as u32,
        ..default()
    };

    // This Image serves as a canvas representing the low-resolution game screen
    let mut canvas = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size: canvas_size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };

    canvas.resize(canvas_size);

    let image_handle = images.add(canvas);

    commands.spawn((
        Camera2d,
        Camera {
            order: -1,
            target: RenderTarget::Image(image_handle.clone().into()),
            clear_color: ClearColorConfig::Custom(GRAY.into()),
            ..default()
        },
        Msaa::Off,
        InGameCamera,
        PIXEL_PERFECT_LAYERS,
    ));

    // spawn the canvas
    commands.spawn((
        Sprite::from_image(image_handle),
        Canvas,
        HIGH_RES_LAYERS,
    ));

    // The "outer" camera renders whatever is on `HIGH_RES_LAYERS` to the screen.
    // here, the canvas and one of the sample sprites will be rendered by this camera
    commands.spawn((
        Camera2d, 
        Msaa::Off, 
        OuterCamera, 
        HIGH_RES_LAYERS
    ));
}

fn setup_physics(mut commands: Commands, assets: Res<AssetServer>) {
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
        Collider::ball(12.0),
        Restitution::coefficient(0.7),
        Transform::from_xyz(0.0, 200.0, 0.0),
        Friction::coefficient(0.5),
        Damping {
            linear_damping: 0.9,
            angular_damping: 0.9,
        },
    ))
    .with_children(|ent|
    {
        ent.spawn(BombPlaceSpotBundle::ball_with_radius(20.0))
        .with_child((
            Sprite::from_color(Color::Srgba(Srgba::RED), [8.0, 8.0].into()),
            Visibility::Hidden,
            Bomb,
            Transform::default(),
        ));
        // ent.spawn((
        //     Sprite::from_color(Color::Srgba(Srgba::RED), [20.0, 20.0].into()),
        //     Visibility::Hidden,
        //     Bomb,
        //     Transform::default(),
        // ));
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
            snap_to_ground: None,
            ..Default::default()
        },
        Sprite::from_image(
            assets.load("guy.png")
        ),
        PIXEL_PERFECT_LAYERS,
        Transform::from_xyz(120.0, 60.0, 0.0),
        Collider::cuboid(6.0, 8.0),
        // Restitution::coefficient(0.5),
        RigidBody::KinematicPositionBased,
        Velocity::default(),
        Friction::coefficient(0.2),
        Damping {
            linear_damping: 0.2,
            angular_damping: 0.2,
        },
    ))
    .with_children(|ent|
    {
        ent.spawn(BombPlacerBundle::ball_with_radius(16.0));
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
    visibility: Visibility
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
                visibility: Visibility::Inherited
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
                visibility: Visibility::Inherited
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
    mut players: Query<(&Velocity, &mut KinematicCharacterController, Option<&KinematicCharacterControllerOutput>), With<Player>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    r_context_mut: Single<&mut RapierContextSimulation>,
    r_config: Single<&RapierConfiguration>,
    time: Res<Time>,
    mut velocity: Local<Velocity>,
)
{
    const PLAYER_ACCEL: f32 = 360.0;
    const PLAYER_DECEL: f32 = 1000.0;
    const PLAYER_MAX_SPEED: f32 = 160.0;

    // println!("Player move system running. Number of player queries found: {:?}", players.iter().count());

    let gravity = r_config.gravity;

    for (vel, mut char, output) in players.iter_mut()
    {
        let ref mut vel = *velocity;
        
        let mut new_vel: f32 = 0.0;

        let left = keyboard.pressed(KeyCode::ArrowLeft);
        let right = keyboard.pressed(KeyCode::ArrowRight);
        if left || right
        {
            let sign: f32 = if left { -1.0 } else { 1.0 };
            let mut acc = PLAYER_ACCEL;
            if vel.linvel.x.signum() != sign.signum()
            {
                acc = f32::max(PLAYER_ACCEL, PLAYER_DECEL);
            }
            acc *= sign;

            new_vel = vel.linvel.x + acc * time.delta_secs();
            new_vel = new_vel.clamp(-PLAYER_MAX_SPEED, PLAYER_MAX_SPEED);

            // new_vel = acc * time.delta_secs();
            // new_vel = new_vel.clamp(-PLAYER_MAX_SPEED, PLAYER_MAX_SPEED);

            // new_vel
        }
        else 
        {
            // let mut new_vel = 0.0;
            // let ref mut vel = *velocity;
            if vel.linvel.x.abs() > 0.0
            {
                let sign = vel.linvel.x.signum();
                new_vel = vel.linvel.x - sign * PLAYER_DECEL * time.delta_secs();
                if new_vel.signum() != sign { new_vel = 0.0; }
                // if new_vel.signum() != sign { new_vel = -vel.linvel.x * 0.2; }
            }    

            // new_vel
        };

        // new_vel = new_vel.clamp(-PLAYER_MAX_SPEED, PLAYER_MAX_SPEED);
        (*velocity).linvel.x = new_vel;

        if let Some(output) = output
        {
            if output.grounded
            {
                (*velocity).linvel.y = 0.0; // Reset vertical velocity when grounded
            }
            else
            {
                (*velocity).linvel += gravity * time.delta_secs();
            }
            // println!("Player output: {:#?}", output);

            if keyboard.just_pressed(KeyCode::ArrowUp) && output.grounded
            {
                println!("Player jumped!");
                (*velocity).linvel.y = 60.0;
            }

        }


        char.translation = Some((*velocity).linvel * time.delta_secs());
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
        ball.linvel.y += 50.0;
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
    bomb_placers: Query<(Entity, Option<&Children>), With<BombPromixityPlacer>>,
    // placer_imgs: Query<(Entity, &Children), With<BombPromixityPlacer>>,
    mut bomb_img: Query<&mut Visibility, With<Bomb>>,
    bomb_place_spots: Query<(Entity, Option<&Children>), With<BombPlaceSpot>>,
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

        let placer = 
        bomb_placers.get(a)
        .or_else(|_| bomb_placers.get(b)).ok();

        let spot =
        bomb_place_spots.get(b)
        .or_else(|_| bomb_place_spots.get(a)).ok();

        #[allow(unused_variables)]
        if let (
            Some((placer, placer_c)), 
            Some((spot, spot_c))
        ) = (placer, spot)
        {
            let Some(spot_c) = spot_c else { continue; };
            for child in spot_c.iter()
            {
                if let Ok(mut vis) = bomb_img.get_mut(child)
                {
                    use SensorInteraction::*;
                    match t {
                        Entered => *vis = Visibility::Inherited,
                        Exited => *vis = Visibility::Hidden,
                    }
                }
            }
        }

        // case 3: not interested in this event right now
    }
}

/// Scales camera projection to fit the window (integer multiples only).
fn fit_canvas(
    mut resize_events: EventReader<WindowResized>,
    mut projection: Single<&mut Projection, With<OuterCamera>>,
) 
{
    let Projection::Orthographic(projection) = &mut **projection else {
        return;
    };
    for event in resize_events.read() {
        let h_scale = event.width / RES_WIDTH as f32;
        let v_scale = event.height / RES_HEIGHT as f32;
        projection.scale = 1. / h_scale.min(v_scale).round();
    }
}


fn print_ball_altitude(positions: Query<&Transform, With<Ball>>) 
{
    for transform in positions.iter() {
        // println!("Ball altitude: {}", transform.translation.y);
    }
}
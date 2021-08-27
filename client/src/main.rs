use bevy::{
    input::{keyboard::KeyboardInput, ElementState},
    prelude::*,
    render::camera::{Camera, WindowOrigin},
};

use bevy_spicy_networking::NetworkClient;
use log::LevelFilter;
use player::{Me, PlayerPlugin};
use simple_logger::SimpleLogger;
use std::convert::TryInto;

use network::NetworkPlugin;
use walk_animation::{walk_animation, WalkAnimation};

mod network;
mod player;
mod walk_animation;

use woods_common::{Direction, MoveInput, Position};

struct WalkEvent {
    player: Entity,
    me: bool,
    direction: Direction,
    to: Position,
    distance: u16,
}

impl WalkEvent {
    fn from(
        player: Entity,
        me: bool,
        previous_direction: Direction,
        direction: Direction,
        from: Position,
    ) -> Self {
        let distance = if previous_direction == direction {
            1
        } else {
            // Turning (i.e. changing directions) requires its own keydown
            0
        };
        let translation = direction.translation() * distance as f32;
        let to = Position {
            x: (translation.x + from.x as f32) as u16,
            y: (translation.y + from.y as f32) as u16,
        };

        WalkEvent {
            player,
            me,
            direction,
            to,
            distance,
        }
    }

    fn should_animate(&self) -> bool {
        self.distance > 0
    }
}

const SCREEN_WIDTH: f32 = 600.0;
const SCREEN_HEIGHT: f32 = 400.0;

fn main() {
    SimpleLogger::new()
        .with_level(LevelFilter::Off)
        .with_module_level("woods_client", LevelFilter::Trace)
        .init()
        .unwrap();

    App::build()
        .insert_resource(WindowDescriptor {
            title: "Woods".to_string(),
            width: SCREEN_WIDTH,
            height: SCREEN_HEIGHT,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(NetworkPlugin)
        .add_plugin(PlayerPlugin)
        .add_startup_system(setup_camera.system())
        .add_startup_system(setup_background.system())
        .add_system(keyboard_movement.system())
        .add_system(walk.system())
        .add_system(create_offset_parent.system())
        .add_system(walk_animation.system().label("walk_animation"))
        .add_system(camera_movement.system().after("walk_animation"))
        .add_system_to_stage(CoreStage::PostUpdate, perspective.system())
        .add_event::<WalkEvent>()
        .run();
}

fn setup_camera(mut commands: Commands) {
    let mut camera = OrthographicCameraBundle::new_2d();
    camera.orthographic_projection.window_origin = WindowOrigin::BottomLeft;
    commands.spawn_bundle(camera);
}

const MAP_WIDTH: f32 = 1000.0;
const MAP_HEIGHT: f32 = 1000.0;

fn setup_background(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let texture_handle = asset_server.load("field.png");
    let sprite_bundle = SpriteBundle {
        material: materials.add(texture_handle.into()),
        ..Default::default()
    };
    commands
        .spawn_bundle(sprite_bundle)
        .insert(TransformOffset(Transform::from_translation(Vec3::new(
            MAP_WIDTH / 2.0,
            MAP_HEIGHT / 2.0,
            0.0,
        ))));
}

struct TransformOffset(pub Transform);

fn create_offset_parent(
    mut commands: Commands,
    mut query: Query<(Entity, &TransformOffset), Without<Parent>>,
) {
    for (entity, transform_offset) in query.iter_mut() {
        commands
            .spawn()
            .insert(transform_offset.0)
            .insert(GlobalTransform::default())
            .push_children(&[entity])
            .id();
    }
}

fn keyboard_movement(
    mut keyboard_input_events: EventReader<KeyboardInput>,
    mut query: Query<(Entity, &WalkAnimation, &Position, &Direction), With<Me>>,
    mut walk_events: EventWriter<WalkEvent>,
) {
    for event in keyboard_input_events
        .iter()
        .filter(|e| e.state == ElementState::Pressed)
    {
        if let Some(key_code) = event.key_code {
            if let Ok(to_direction) = key_code.try_into() {
                for (entity, walk_animation, position, direction) in query.iter_mut() {
                    // Ignore keys until walk animation finishes
                    if walk_animation.running() {
                        continue;
                    }

                    walk_events.send(WalkEvent::from(
                        entity,
                        true,
                        *direction,
                        to_direction,
                        *position,
                    ));
                }
            }
        }
    }
}

fn camera_movement(
    mut commands: Commands,
    me_query: Query<&Transform, (With<Me>, Changed<Transform>)>,
    camera_query: Query<Entity, With<Camera>>,
) {
    if let Ok(transform) = me_query.single() {
        let camera = camera_query.single().unwrap();

        let mut camera_transform = Transform::from_translation(
            transform.translation - Vec3::new(SCREEN_WIDTH / 2.0, SCREEN_HEIGHT / 2.0, 0.0),
        );

        camera_transform.translation.x = camera_transform
            .translation
            .x
            .clamp(0.0, MAP_WIDTH - SCREEN_WIDTH);
        camera_transform.translation.y = camera_transform
            .translation
            .y
            .clamp(0.0, MAP_HEIGHT - SCREEN_HEIGHT);

        camera_transform.translation.z = 999.0;

        commands.entity(camera).insert(camera_transform);
    }
}

#[derive(Default)]
struct Collide;

fn walk(
    mut walk_events: EventReader<WalkEvent>,
    net: Res<NetworkClient>,
    mut commands: Commands,
    query: Query<(&Collide, &Position), Without<Me>>,
) {
    for walk_event in walk_events.iter() {
        if walk_event.me {
            let collision = query.iter().any(|(_, position)| *position == walk_event.to);
            if collision {
                log::trace!("Ignoring move attempt due to collision");
                continue;
            }
        }

        let mut entity_commands = commands.entity(walk_event.player);

        entity_commands
            .insert(walk_event.direction)
            .insert(walk_event.to);

        if walk_event.should_animate() {
            entity_commands.insert(WalkAnimation::new());
        }

        if walk_event.me {
            net.send_message(MoveInput(walk_event.direction, walk_event.to))
                .unwrap();
        }
    }
}

fn perspective(mut query: Query<(&mut Transform, &Position)>) {
    // Sprites should render top-to-bottom so things lower down overlap things higher up
    for (mut transform, position) in query.iter_mut() {
        let far = 999; // camera is at 1000; see OrthographicCameraBundle
        transform.translation.z = (far - position.y).into();
    }
}

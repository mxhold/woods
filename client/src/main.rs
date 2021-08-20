use bevy::{input::{keyboard::KeyboardInput, ElementState}, prelude::*, render::camera::WindowOrigin};

use bevy_networking_turbulence::NetworkResource;
use log::LevelFilter;
use player::{Me, PlayerPlugin};
use simple_logger::SimpleLogger;
use std::convert::TryInto;

use network::NetworkPlugin;
use walk_animation::{walk_animation, WalkAnimation};

mod network;
mod player;
mod walk_animation;

use woods_common::{ClientMessage, Direction, Position};

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
        let distance = Self::distance(previous_direction, direction);
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

    fn distance(previous_direction: Direction, direction: Direction) -> u16 {
        if previous_direction == direction {
            1
        } else {
            // Turning (i.e. changing directions) requires its own keydown
            0
        }
    }

    fn should_animate(&self) -> bool {
        self.distance > 0
    }
}

fn main() {
    SimpleLogger::new()
        .with_level(LevelFilter::Off)
        .with_module_level("bevy_networking_turbulence", LevelFilter::Trace)
        .with_module_level("woods_client", LevelFilter::Trace)
        .init()
        .unwrap();

    App::build()
        .insert_resource(WindowDescriptor {
            title: "Woods".to_string(),
            width: 400.0,
            height: 300.0,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(NetworkPlugin)
        .add_plugin(PlayerPlugin)
        .add_startup_system(setup_camera.system())
        .add_system(keyboard_movement.system())
        .add_system(walk.system())
        .add_system(walk_animation.system())
        .add_event::<WalkEvent>()
        .run();
}

fn setup_camera(mut commands: Commands) {
    let mut camera = OrthographicCameraBundle::new_2d();
    camera.orthographic_projection.window_origin = WindowOrigin::BottomLeft;
    commands.spawn_bundle(camera);
}

fn keyboard_movement(
    mut keyboard_input_events: EventReader<KeyboardInput>,
    mut query: Query<(Entity, &Me, &WalkAnimation, &Position, &Direction)>,
    mut walk_events: EventWriter<WalkEvent>,
) {
    for event in keyboard_input_events
        .iter()
        .filter(|e| e.state == ElementState::Pressed)
    {
        if let Some(key_code) = event.key_code {
            if let Ok(to_direction) = key_code.try_into() {
                for (entity, _, walk_animation, position, direction) in query.iter_mut() {
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

fn walk(
    mut walk_events: EventReader<WalkEvent>,
    mut net: ResMut<NetworkResource>,
    mut commands: Commands,
) {
    for walk_event in walk_events.iter() {
        let mut entity_commands = commands.entity(walk_event.player);

        entity_commands
            .insert(walk_event.direction)
            .insert(walk_event.to);

        if walk_event.should_animate() {
            entity_commands.insert(WalkAnimation::new());
        }

        if walk_event.me {
            log::info!("Broadcasting move {:?}", walk_event.direction);
            net.broadcast_message(ClientMessage::Move(
                walk_event.direction.into(),
                walk_event.to,
            ));
        }
    }
}

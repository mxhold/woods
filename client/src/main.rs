use bevy::{
    input::{keyboard::KeyboardInput, ElementState},
    prelude::*,
    render::camera::WindowOrigin,
};

use bevy_networking_turbulence::NetworkResource;
use log::LevelFilter;
use simple_logger::SimpleLogger;
use std::convert::TryInto;

use walk_animation::{WalkAnimation, walk_animation};
use network::NetworkPlugin;

mod network;
mod walk_animation;

use woods_common::{ClientMessage, Position, Direction};

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
        .add_startup_system(setup_me.system())
        .add_system(keyboard_movement.system())
        .add_system(walk.system())
        .add_system(walk_animation.system())
        .add_event::<WalkEvent>()
        .run();
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

struct Me;

#[derive(Bundle)]
struct PlayerBundle {
    #[bundle]
    sprite_sheet: SpriteSheetBundle,
    direction: Direction,
    walk_animation: WalkAnimation,
}

impl Default for PlayerBundle {
    fn default() -> Self {
        Self {
            sprite_sheet: SpriteSheetBundle {
                transform: Transform::from_xyz(10., 20., 0.),
                ..Default::default()
            },
            direction: Direction::South,
            walk_animation: Default::default(),
        }
    }
}

fn setup_me(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let texture_handle = asset_server.load("player.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(19.0, 38.0), 24, 1);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    let mut camera = OrthographicCameraBundle::new_2d();
    camera.orthographic_projection.window_origin = WindowOrigin::BottomLeft;
    commands.spawn_bundle(camera);
    commands
        .spawn_bundle(PlayerBundle {
            sprite_sheet: SpriteSheetBundle {
                texture_atlas: texture_atlas_handle,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Me);
}

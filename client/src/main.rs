use bevy::{
    input::{keyboard::KeyboardInput, ElementState},
    prelude::*,
    render::camera::WindowOrigin,
};
use bevy_networking_turbulence::{
    ConnectionChannelsBuilder, NetworkEvent, NetworkResource, NetworkingPlugin,
};
use log::LevelFilter;
use simple_logger::SimpleLogger;
use std::{collections::HashMap, convert::TryInto, net::SocketAddr};

use direction::Direction;
use walk_animation::WalkAnimation;

mod direction;
mod walk_animation;

use woods_common::{
    ClientMessage, PlayerId, Position, ServerMessage, CLIENT_MESSAGE_SETTINGS,
    SERVER_MESSAGE_SETTINGS, SERVER_PORT,
};

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
            // Changing directions requires its own keydown
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
        .insert_resource(Players::default())
        .add_plugins(DefaultPlugins)
        .add_plugin(NetworkingPlugin::default())
        .add_startup_system(setup_me.system())
        .add_startup_system(connect.system())
        .add_startup_system(network_setup.system())
        .add_system(keyboard_movement.system())
        .add_system(walk.system())
        .add_system(walk_animation.system())
        .add_system(handle_network_connections.system())
        .add_system(handle_messages.system())
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

fn walk_animation(
    time: Res<Time>,
    mut query: Query<(
        &mut TextureAtlasSprite,
        &Direction,
        &mut WalkAnimation,
        &Position,
        &mut Transform,
    )>,
) {
    for (mut sprite, direction, mut walk_animation, position, mut transform) in query.iter_mut() {
        walk_animation.tick(time.delta());

        sprite.index = walk_animation.sprite_index_offset() + direction.sprite_index_offset();

        transform.translation = walk_animation.translate(position, direction);
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

fn connect(mut net: ResMut<NetworkResource>) {
    let ip_address =
        bevy_networking_turbulence::find_my_ip_address().expect("can't find ip address");
    let socket_address = SocketAddr::new(ip_address, SERVER_PORT);
    log::info!("Starting client");
    net.connect(socket_address);
}

fn network_setup(mut net: ResMut<NetworkResource>) {
    net.set_channels_builder(|builder: &mut ConnectionChannelsBuilder| {
        builder
            .register::<ClientMessage>(CLIENT_MESSAGE_SETTINGS)
            .unwrap();
        builder
            .register::<ServerMessage>(SERVER_MESSAGE_SETTINGS)
            .unwrap();
    });
}

#[derive(Default)]
struct Players(pub HashMap<PlayerId, Entity>);

fn handle_messages(
    mut net: ResMut<NetworkResource>,
    me_query: Query<Entity, With<Me>>,
    mut commands: Commands,
    mut players: ResMut<Players>,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut walk_events: EventWriter<WalkEvent>,
) {
    for (handle, connection) in net.connections.iter_mut() {
        let channels = connection.channels().unwrap();

        while let Some(server_message) = channels.recv::<ServerMessage>() {
            log::debug!(
                "ServerMessage received on [{}]: {:?}",
                handle,
                server_message
            );

            let me = me_query.single().unwrap();

            match server_message {
                ServerMessage::Hello(player_id, position) => {
                    log::trace!("My id is {:?}. I'm at {:?}.", player_id, position);
                    commands.entity(me).insert(player_id).insert(position);
                    players.0.insert(player_id, me);
                }
                ServerMessage::Move(player_id, direction, position) => {
                    // TODO: the fact that you can skip this line and not get a compiler error
                    // makes me want to try to find some way to avoid setting components of unknown types!
                    let direction: Direction = direction.into();

                    log::debug!("{:?} moved {:?} to {:?}", player_id, direction, position);

                    match players.0.get(&player_id) {
                        Some(player) => {
                            if player.id() == me.id() {
                                log::trace!("Skipping move for self");
                                // TODO: if the position from the server doesn't match what we have, correct it
                                continue;
                            }

                            walk_events.send(WalkEvent {
                                player: *player,
                                me: false,
                                direction,
                                to: position,
                                distance: 1, // TODO: get from server
                            });
                        }
                        None => {
                            log::debug!(
                                "New player seen {:?} at {:?} facing {:?}",
                                player_id,
                                position,
                                direction
                            );
                            let player = commands.spawn().id();
                            let texture_handle = asset_server.load("player.png");
                            let texture_atlas = TextureAtlas::from_grid(
                                texture_handle,
                                Vec2::new(19.0, 38.0),
                                24,
                                1,
                            );
                            let texture_atlas_handle = texture_atlases.add(texture_atlas);
                            commands
                                .entity(player)
                                .insert_bundle(PlayerBundle {
                                    sprite_sheet: SpriteSheetBundle {
                                        texture_atlas: texture_atlas_handle,
                                        ..Default::default()
                                    },
                                    walk_animation: WalkAnimation::new(),
                                    ..Default::default()
                                })
                                .insert(direction)
                                .insert(position);
                            players.0.insert(player_id, player);
                            walk_events.send(WalkEvent {
                                player,
                                me: false,
                                direction,
                                to: position,
                                distance: 1, // TODO: get from server
                            });
                        }
                    }
                }
            }
        }
    }
}

fn handle_network_connections(
    mut net: ResMut<NetworkResource>,
    mut network_events: EventReader<NetworkEvent>,
) {
    for event in network_events.iter() {
        match event {
            NetworkEvent::Connected(handle) => match net.connections.get_mut(handle) {
                Some(connection) => {
                    match connection.remote_address() {
                        Some(remote_address) => {
                            log::info!(
                                "Incoming connection on [{}] from [{}]",
                                handle,
                                remote_address
                            );
                        }
                        None => {
                            log::info!("Connected on [{}]", handle);
                        }
                    };

                    // Gotta send something for the server to recognize the client has connected.
                    // TODO: understand why this is necessary
                    net.broadcast_message(ClientMessage::Hello);
                }
                None => panic!("Got packet for non-existing connection [{}]", handle),
            },
            _ => {}
        }
    }
}

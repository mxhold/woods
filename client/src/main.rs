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
        .add_system(walk_animation.system())
        .add_system(handle_network_connections.system())
        .add_system_to_stage(CoreStage::PostUpdate, handle_messages.system())
        .run();
}

fn keyboard_movement(
    mut keyboard_input_events: EventReader<KeyboardInput>,
    mut query: Query<(&Me, &mut Direction, &mut WalkAnimation, &mut Position)>,
    mut net: ResMut<NetworkResource>,
) {
    for event in keyboard_input_events
        .iter()
        .filter(|e| e.state == ElementState::Pressed)
    {
        if let Some(key_code) = event.key_code {
            if let Ok(to_direction) = key_code.try_into() {
                for (_, direction, walk_animation, position) in query.iter_mut() {
                    start_walking(to_direction, direction, walk_animation, position, &mut net);
                }
            }
        }
    }
}

fn start_walking(
    to_direction: Direction,
    mut direction: Mut<Direction>,
    mut walk_animation: Mut<WalkAnimation>,
    mut position: Mut<Position>,
    net: &mut ResMut<NetworkResource>,
) {
    if walk_animation.running() {
        return;
    }

    if to_direction == *direction {
        let translation = to_direction.translation();
        position.x = (translation.x + position.x as f32) as u16;
        position.y = (translation.y + position.y as f32) as u16;

        *walk_animation = WalkAnimation::new();
    } else {
        // Don't move if just changing directions
        *direction = to_direction;
    }

    log::info!("Broadcasting move {:?}", to_direction);
    net.broadcast_message(ClientMessage::Move(to_direction.into(), *position));
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
                    log::debug!("{:?} moved {:?} to {:?}", player_id, direction, position);

                    match players.0.get(&player_id) {
                        Some(player) => {
                            if player.id() == me.id() {
                                log::trace!("Skipping move for self");
                                // TODO: do more?
                                continue;
                            }

                            log::debug!("Move {:?} {:?} to {:?}", player_id, direction, position);
                            commands
                                .entity(*player)
                                .insert(position)
                                .insert(direction)
                                .insert(WalkAnimation::new());
                        }
                        None => {
                            log::debug!("New player seen {:?} at {:?} facing {:?}", player_id, position, direction);
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

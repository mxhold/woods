use std::{collections::HashMap, net::SocketAddr};

use bevy::prelude::*;
use bevy_networking_turbulence::{
    ConnectionChannelsBuilder, NetworkEvent, NetworkResource, NetworkingPlugin,
};
use woods_common::{
    ClientMessage, PlayerId, ServerMessage, CLIENT_MESSAGE_SETTINGS, SERVER_MESSAGE_SETTINGS,
    SERVER_PORT
};

use crate::{walk_animation::WalkAnimation, Me, PlayerBundle, WalkEvent};

#[derive(Default)]
struct Players(pub HashMap<PlayerId, Entity>);

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugin(NetworkingPlugin::default())
            .insert_resource(Players::default())
            .add_startup_system(connect.system())
            .add_startup_system(network_setup.system())
            .add_system(handle_network_connections.system())
            .add_system(handle_messages.system());
    }
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
                ServerMessage::Move {
                    player_id,
                    direction,
                    position,
                    distance,
                } => {
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
                                distance,
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
                                distance,
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

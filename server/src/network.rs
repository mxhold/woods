use bevy::prelude::*;
use std::collections::HashMap;
use std::net::SocketAddr;

use bevy_networking_turbulence::{
    ConnectionChannelsBuilder, NetworkEvent, NetworkResource, NetworkingPlugin,
};
use woods_common::{
    ClientMessage, PlayerId, ServerMessage, CLIENT_MESSAGE_SETTINGS,
    SERVER_MESSAGE_SETTINGS, SERVER_PORT,
};

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugin(NetworkingPlugin::default())
            .add_startup_system(network_setup.system())
            .add_system_to_stage(CoreStage::PreUpdate, handle_messages.system())
            .add_system(handle_network_connections.system())
            .add_system(handle_messages.system())
            .insert_resource(PlayerIds::default())
            .add_event::<PlayerConnected>()
            .add_event::<PlayerMessage>();
    }
}

pub struct PlayerConnected(pub PlayerId, pub Entity);

#[derive(Default)]
struct PlayerIds(pub HashMap<PlayerId, Entity>);

pub struct PlayerMessage {
    pub player: Entity,
    pub client_message: ClientMessage,
}

fn network_setup(mut net: ResMut<NetworkResource>) {
    let ip_address =
        bevy_networking_turbulence::find_my_ip_address().expect("can't find ip address");
    let socket_address = SocketAddr::new(ip_address, SERVER_PORT);
    log::info!("Starting server");
    net.listen(socket_address, None, None);

    net.set_channels_builder(|builder: &mut ConnectionChannelsBuilder| {
        builder
            .register::<ClientMessage>(CLIENT_MESSAGE_SETTINGS)
            .unwrap();
        builder
            .register::<ServerMessage>(SERVER_MESSAGE_SETTINGS)
            .unwrap();
    });
}

fn handle_network_connections(
    mut net: ResMut<NetworkResource>,
    mut network_events: EventReader<NetworkEvent>,
    mut player_connected: EventWriter<PlayerConnected>,
    mut commands: Commands,
    mut players: ResMut<PlayerIds>,
) {
    for event in network_events.iter() {
        match event {
            NetworkEvent::Connected(handle) => match net.connections.get_mut(handle) {
                Some(connection) => {
                    match connection.remote_address() {
                        Some(remote_address) => {
                            log::debug!(
                                "Incoming connection on [{}] from [{}]",
                                handle,
                                remote_address
                            );
                        }
                        None => {
                            log::debug!("Connected on [{}]", handle);
                        }
                    }
                    let player = commands.spawn().id();
                    let player_id = PlayerId(*handle);
                    players.0.insert(player_id, player);
                    player_connected.send(PlayerConnected(player_id, player));
                }
                None => panic!("Got packet for non-existing connection [{}]", handle),
            },
            _ => {}
        }
    }
}


fn handle_messages(
    mut net: ResMut<NetworkResource>,
    players: Res<PlayerIds>,
    mut player_messages: EventWriter<PlayerMessage>,
) {
    for (handle, connection) in net.connections.iter_mut() {
        let channels = connection.channels().unwrap();
        while let Some(client_message) = channels.recv::<ClientMessage>() {
            log::debug!("Received [{}]: {:?}", handle, client_message);

            if let ClientMessage::Hello = client_message {
                // Player won't exist yet; also this message is just to trigger the connection so nothing to do anyway.
                continue;
              }

            let player = *players
                .0
                .get(&PlayerId(*handle))
                .expect("no player with handle");

            player_messages.send(PlayerMessage {
                player,
                client_message,
            });
        }
    }
}

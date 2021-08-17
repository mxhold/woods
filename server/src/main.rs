use std::collections::HashMap;
use std::{net::SocketAddr, time::Duration};

use bevy::{app::ScheduleRunnerSettings, prelude::*};
use bevy_networking_turbulence::{
    ConnectionChannelsBuilder, NetworkEvent, NetworkResource, NetworkingPlugin,
};
use log::LevelFilter;
use simple_logger::SimpleLogger;
use woods_common::{
    ClientMessage, PlayerId, Position, ServerMessage, CLIENT_STATE_MESSAGE_SETTINGS,
    SERVER_MESSAGE_SETTINGS, SERVER_PORT,
};

#[derive(Default)]
struct Players(pub HashMap<PlayerId, Entity>);

#[derive(Default)]
struct ServerMessages(pub Vec<(u32, ServerMessage)>);

fn main() {
    SimpleLogger::new()
        .with_level(LevelFilter::Off)
        .with_module_level("woods_server", LevelFilter::Trace)
        .init()
        .unwrap();

    App::build()
        .insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_secs_f64(
            1.0 / 60.0,
        )))
        .add_plugins(MinimalPlugins)
        .add_plugin(NetworkingPlugin::default())
        .add_startup_system(setup.system())
        .add_startup_system(network_setup.system())
        .add_system_to_stage(CoreStage::PreUpdate, handle_messages_server.system())
        .add_system_to_stage(CoreStage::PostUpdate, broadcast_moves.system())
        .add_system(handle_packets.system())
        .add_system(send_messages.system())
        .insert_resource(Players::default())
        .insert_resource(ServerMessages::default())
        .run();
}

fn setup(mut net: ResMut<NetworkResource>) {
    let ip_address =
        bevy_networking_turbulence::find_my_ip_address().expect("can't find ip address");
    let socket_address = SocketAddr::new(ip_address, SERVER_PORT);
    log::info!("Starting server");
    net.listen(socket_address, None, None);
}

fn network_setup(mut net: ResMut<NetworkResource>) {
    net.set_channels_builder(|builder: &mut ConnectionChannelsBuilder| {
        builder
            .register::<ClientMessage>(CLIENT_STATE_MESSAGE_SETTINGS)
            .unwrap();
        builder
            .register::<ServerMessage>(SERVER_MESSAGE_SETTINGS)
            .unwrap();
    });
}

fn handle_packets(mut net: ResMut<NetworkResource>, mut network_events: EventReader<NetworkEvent>) {
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
                    net.send_message(*handle, ServerMessage::PlayerId(PlayerId(*handle)))
                        .expect("Message failed");
                }
                None => panic!("Got packet for non-existing connection [{}]", handle),
            },
            _ => {}
        }
    }
}

fn handle_messages_server(
    mut net: ResMut<NetworkResource>,
    mut commands: Commands,
    mut players: ResMut<Players>,
    mut messages: ResMut<ServerMessages>,
) {    
    for (handle, connection) in net.connections.iter_mut() {
        let channels = connection.channels().unwrap();
        while let Some(client_message) = channels.recv::<ClientMessage>() {
            log::debug!(
                "ClientMessage received on [{}]: {:?}",
                handle,
                client_message
            );
            match client_message {
                ClientMessage::Move(position) => {
                    log::info!("recv [{}] mov {:?}", handle, position);
                    let player = players.0.get(&PlayerId(*handle)).expect("no player with handle");
                    commands.entity(*player).insert(position);
                }
                ClientMessage::Hello => {
                    log::info!("hello {:?}", handle);
                    let player_id = PlayerId(*handle);
                    let player = commands.spawn().id();
                    commands.entity(player).insert(player_id);
                    players.0.insert(player_id, player);
                    messages.0.push((*handle, ServerMessage::PlayerId(player_id)));
                }
            }
        }
    }
}

fn send_messages(
    mut net: ResMut<NetworkResource>,
    mut messages: ResMut<ServerMessages>
) {
    for (handle, message) in messages.0.drain(..) {
        log::info!("send {:?}", message);
        net.send_message(handle, message).unwrap();
    }
}

fn broadcast_moves(mut net: ResMut<NetworkResource>, query: Query<(&PlayerId, &Position), Changed<Position>>) {
    for (player_id, position) in query.iter() {
        log::info!("broadcasting position {:?} at {:?}", player_id, position);
        net.broadcast_message(ServerMessage::Position(*player_id, *position));
    }
}

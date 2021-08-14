use std::{net::SocketAddr, time::Duration};

use bevy::{app::ScheduleRunnerSettings, prelude::*};
use bevy_networking_turbulence::{
    ConnectionChannelsBuilder, NetworkEvent,
    NetworkResource, NetworkingPlugin,
};
use serde::{Deserialize, Serialize};
use simple_logger::SimpleLogger;
use woods_common::{CLIENT_STATE_MESSAGE_SETTINGS, SERVER_MESSAGE_SETTINGS, SERVER_PORT};

fn main() {
    SimpleLogger::new()
        .init()
        .expect("A logger was already initialized");

    App::build()
        .insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_secs_f64(
            1.0 / 60.0,
        )))
        .add_plugins(MinimalPlugins)
        .add_plugin(NetworkingPlugin::default())
        .add_startup_system(setup.system())
        .add_system_to_stage(CoreStage::PreUpdate, handle_messages_server.system())
        .add_startup_system(network_setup.system())
        .add_system(handle_packets.system())
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

#[derive(Serialize, Deserialize, Debug, Clone)]
enum ClientMessage {
    Hello(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum ServerMessage {
    Welcome(String),
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
                    net.send_message(*handle, ServerMessage::Welcome("welcome!".to_owned()))
                        .expect("Message failed");
                }
                None => panic!("Got packet for non-existing connection [{}]", handle),
            },
            _ => {}
        }
    }
}

fn handle_messages_server(mut net: ResMut<NetworkResource>) {
    for (handle, connection) in net.connections.iter_mut() {
        let channels = connection.channels().unwrap();
        while let Some(client_message) = channels.recv::<ClientMessage>() {
            log::debug!(
                "ClientMessage received on [{}]: {:?}",
                handle,
                client_message
            );
            match client_message {
                ClientMessage::Hello(id) => {
                    log::info!("Client [{}] connected on [{}]", id, handle);
                }
            }
        }
    }
}

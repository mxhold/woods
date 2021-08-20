use std::collections::HashMap;
use std::{net::SocketAddr, time::Duration};

use bevy::{app::ScheduleRunnerSettings, prelude::*};
use bevy_networking_turbulence::{
    ConnectionChannelsBuilder, NetworkEvent, NetworkResource, NetworkingPlugin,
};
use log::LevelFilter;
use rand::{thread_rng, Rng};
use simple_logger::SimpleLogger;
use woods_common::{
    ClientMessage, Direction, PlayerId, Position, ServerMessage, CLIENT_MESSAGE_SETTINGS,
    SERVER_MESSAGE_SETTINGS, SERVER_PORT,
};

struct PlayerConnected(PlayerId);

#[derive(Default)]
struct PlayerIds(pub HashMap<PlayerId, Entity>);

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
        .add_startup_system(network_setup.system())
        .add_system_to_stage(CoreStage::PreUpdate, handle_messages.system())
        .add_system(handle_network_connections.system())
        .add_system(handle_connections.system())
        .add_system(broadcast_moves.system())
        .insert_resource(PlayerIds::default())
        .add_event::<PlayerConnected>()
        .add_event::<MoveEvent>()
        .run();
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
    mut ev_player_connected: EventWriter<PlayerConnected>,
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
                    ev_player_connected.send(PlayerConnected(PlayerId(*handle)));
                }
                None => panic!("Got packet for non-existing connection [{}]", handle),
            },
            _ => {}
        }
    }
}

fn random_position() -> Position {
    let mut rng = thread_rng();
    let x: u16 = rng.gen_range(0..16);
    let y: u16 = rng.gen_range(0..16);

    Position { x, y }
}

fn handle_connections(
    mut net: ResMut<NetworkResource>,
    mut ev_player_connected: EventReader<PlayerConnected>,
    mut commands: Commands,
    mut players: ResMut<PlayerIds>,
) {
    for PlayerConnected(player_id) in ev_player_connected.iter() {
        let player = commands.spawn().id();
        let position = random_position();
        commands
            .entity(player)
            .insert(player_id.clone())
            .insert(Direction::South)
            .insert(position);
        players.0.insert(*player_id, player);

        net.send_message(player_id.0, ServerMessage::Hello(*player_id, position))
            .expect("Hello failed");
    }
}

fn handle_messages(
    mut net: ResMut<NetworkResource>,
    players: Res<PlayerIds>,
    mut move_events: EventWriter<MoveEvent>,
) {
    for (handle, connection) in net.connections.iter_mut() {
        let channels = connection.channels().unwrap();
        while let Some(client_message) = channels.recv::<ClientMessage>() {
            log::debug!("RECV [{}]: {:?}", handle, client_message);
            match client_message {
                ClientMessage::Move(direction, position) => {
                    let player = *players
                        .0
                        .get(&PlayerId(*handle))
                        .expect("no player with handle");

                    move_events.send(MoveEvent {
                        player,
                        direction,
                        position,
                    });
                }
                ClientMessage::Hello => {
                    // Nothing to do -- the client just sends this to start the connection
                }
            }
        }
    }
}

fn broadcast_moves(
    mut commands: Commands,
    mut query: Query<(&Position, &Direction, &PlayerId)>,
    mut move_events: EventReader<MoveEvent>,
    mut net: ResMut<NetworkResource>,
) {
    for move_event in move_events.iter() {
        if let Ok((_current_position, current_direction, player_id)) =
            query.get_mut(move_event.player)
        {
            let distance: u16;

            if *current_direction != move_event.direction {
                // Player is just turning
                commands
                    .entity(move_event.player)
                    .insert(move_event.direction);
                distance = 0;
            } else {
                // TODO: validate new position is adjacent to existing position
                commands
                    .entity(move_event.player)
                    .insert(move_event.position);
                distance = 1;
            }

            broadcast(
                &mut net,
                ServerMessage::Move {
                    player_id: *player_id,
                    direction: move_event.direction,
                    position: move_event.position,
                    distance,
                },
            );
        } else {
            log::warn!("Ignoring Move for player without direction/position");
        }
    }
}

struct MoveEvent {
    direction: Direction,
    position: Position,
    player: Entity,
}

fn broadcast(net: &mut NetworkResource, message: ServerMessage) {
    log::debug!("BROADCAST {:?}", message);
    net.broadcast_message(message);
}

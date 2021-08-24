use bevy::prelude::*;
use bevy_spicy_networking::{
    AppNetworkServerMessage, ConnectionId, NetworkData, NetworkServer, ServerNetworkEvent,
};
use rand::{thread_rng, Rng};
use std::collections::HashMap;
use std::net::SocketAddr;

use woods_common::{Direction, MoveInput, MoveUpdate, PlayerId, PlayerLeft, Position, SERVER_PORT, Welcome};

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugin(bevy_spicy_networking::ServerPlugin)
            .add_startup_system(setup_networking.system())
            .add_system_to_stage(CoreStage::PreUpdate, handle_moves.system())
            .add_system(handle_network_connections.system())
            .add_system(handle_disconnects.system())
            .insert_resource(Players::default())
            .listen_for_server_message::<MoveInput>();
    }
}

#[derive(Default)]
struct Players(pub HashMap<ConnectionId, Entity>);

fn setup_networking(mut net: ResMut<NetworkServer>) {
    let ip_address = "127.0.0.1".parse().unwrap();

    let socket_address = SocketAddr::new(ip_address, SERVER_PORT);

    match net.listen(socket_address) {
        Ok(_) => (),
        Err(err) => {
            log::error!("Could not start listening: {}", err);
            panic!();
        }
    }

    log::info!("Listening on {:?}", socket_address);
}

fn random_position() -> Position {
    let mut rng = thread_rng();
    let x: u16 = rng.gen_range(0..16);
    let y: u16 = rng.gen_range(0..16);

    Position { x, y }
}

fn handle_network_connections(
    mut commands: Commands,
    mut network_events: EventReader<ServerNetworkEvent>,
    mut players: ResMut<Players>,
    net: Res<NetworkServer>,
    query: Query<(&Position, &Direction, &PlayerId, &ConnectionId)>,
    mut next_player_id: Local<u32>,
) {
    for event in network_events.iter() {
        if let ServerNetworkEvent::Connected(connection_id) = event {
            log::debug!("New connection from {:?}", connection_id);
            let player = commands.spawn().id();
            players.0.insert(*connection_id, player);
            *next_player_id += 1;
            let player_id = PlayerId(*next_player_id);
            let direction: Direction = Default::default();
            let position = random_position();
            commands
                .entity(player)
                .insert(player_id)
                .insert(*connection_id)
                .insert(direction)
                .insert(position);

            log::debug!("Hello {:?} @ {:?}", player_id, position);

            net.send_message(*connection_id, Welcome(player_id, position))
                .unwrap();

            for (
                other_player_position,
                other_player_direction,
                other_player_id,
                other_player_connection_id,
            ) in query.iter()
            {
                if *other_player_id == player_id {
                    continue;
                }

                // Send new player position to all other players
                net.send_message(
                    *other_player_connection_id,
                    MoveUpdate {
                        player_id,
                        direction,
                        position,
                        distance: 0,
                    },
                )
                .unwrap();

                // Send positions of all previously connected players to new player
                net.send_message(
                    *connection_id,
                    MoveUpdate {
                        player_id: *other_player_id,
                        direction: *other_player_direction,
                        position: *other_player_position,
                        distance: 0,
                    },
                )
                .unwrap();
            }
        }
    }
}

fn handle_disconnects(
    mut players: ResMut<Players>,
    mut network_events: EventReader<ServerNetworkEvent>,
    query: Query<&PlayerId>,
    mut commands: Commands,
    net: Res<NetworkServer>,
) {
    for event in network_events.iter() {
        if let ServerNetworkEvent::Disconnected(connection_id) = event {
            if let Some(player) = players.0.remove(connection_id) {
                match query.get(player) {
                    Ok(player_id) => {
                        log::info!("{:?} disconnected.", player_id);
                        net.broadcast(PlayerLeft(*player_id));
                    },
                    Err(_) => {
                        log::warn!("Disconnect for player without PlayerId {:?}", connection_id);
                    },
                }
                commands.entity(player).despawn();
            } else {
                log::warn!("Disconnect for connection missing from connections {:?}", connection_id);
            }
        }
    }
}

fn handle_moves(
    players: Res<Players>,
    net: Res<NetworkServer>,
    mut move_inputs: EventReader<NetworkData<MoveInput>>,
    mut query: Query<(&Position, &Direction, &PlayerId)>,
    mut commands: Commands,
) {
    for move_input in move_inputs.iter() {
        let MoveInput(direction, position) = **move_input;

        let player = players
            .0
            .get(&move_input.source())
            .expect("No player associated with connection");

        if let Ok((_current_position, current_direction, player_id)) = query.get_mut(*player) {
            let distance: u16;

            if *current_direction != direction {
                // Player is just turning
                commands.entity(*player).insert(direction);
                distance = 0;
            } else {
                // TODO: validate new position is adjacent to existing position
                // TODO: collision detection
                commands.entity(*player).insert(position);
                distance = 1;
            }

            log::trace!("{:?} moved {:?} {:?} to {:?}", player_id, direction, distance, position);
            net.broadcast(MoveUpdate {
                player_id: *player_id,
                direction: direction,
                position: position,
                distance,
            })
        } else {
            log::warn!("Ignoring Move for player without direction/position");
        }
    }
}

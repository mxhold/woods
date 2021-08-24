use bevy::prelude::*;
use bevy_spicy_networking::{
    AppNetworkServerMessage, ConnectionId, NetworkData, NetworkServer, ServerNetworkEvent,
};
use rand::{thread_rng, Rng};
use std::collections::HashMap;
use std::net::SocketAddr;

use woods_common::{Direction, MoveInput, MoveUpdate, PlayerId, Position, Welcome, SERVER_PORT};

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugin(bevy_spicy_networking::ServerPlugin)
            .add_startup_system(setup_networking.system())
            .add_system_to_stage(CoreStage::PreUpdate, handle_messages.system())
            .add_system(handle_network_connections.system())
            .insert_resource(Connections::default())
            .listen_for_server_message::<MoveInput>();
    }
}

#[derive(Default)]
struct Connections(pub HashMap<ConnectionId, Entity>);

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
    mut connections: ResMut<Connections>,
    net: Res<NetworkServer>,
    query: Query<(&Position, &Direction, &PlayerId, &ConnectionId)>,
    mut next_player_id: Local<u32>,
) {
    for event in network_events.iter() {
        if let ServerNetworkEvent::Connected(connection_id) = event {
            log::debug!("New connection from {:?}", connection_id);
            let player = commands.spawn().id();
            connections.0.insert(*connection_id, player);
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

fn handle_messages(
    connections: Res<Connections>,
    net: Res<NetworkServer>,
    mut move_inputs: EventReader<NetworkData<MoveInput>>,
    mut query: Query<(&Position, &Direction, &PlayerId)>,
    mut commands: Commands,
) {
    for move_input in move_inputs.iter() {
        let MoveInput(direction, position) = **move_input;

        let player = connections
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

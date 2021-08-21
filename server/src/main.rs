use std::time::Duration;

use bevy::{app::ScheduleRunnerSettings, prelude::*};
use bevy_networking_turbulence::NetworkResource;
use log::LevelFilter;
use network::{NetworkPlugin, PlayerConnected, PlayerMessage};
use simple_logger::SimpleLogger;
use woods_common::{ClientMessage, Direction, PlayerId, Position, ServerMessage};
use rand::{thread_rng, Rng};

mod network;

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
        .add_plugin(NetworkPlugin)
        .add_system(broadcast_moves.system())
        .add_system(handle_connections.system())

        .run();
}


fn random_position() -> Position {
    let mut rng = thread_rng();
    let x: u16 = rng.gen_range(0..16);
    let y: u16 = rng.gen_range(0..16);

    Position { x, y }
}

fn handle_connections(
    mut net: ResMut<NetworkResource>,
    mut player_connected: EventReader<PlayerConnected>,
    mut commands: Commands,
    query: Query<(&Position, &Direction, &PlayerId)>
) {
    for PlayerConnected(player_id, player) in player_connected.iter() {
        let direction: Direction = Default::default();
        let position = random_position();
        commands
            .entity(*player)
            .insert(player_id.clone())
            .insert(direction)
            .insert(position);

        log::debug!("Hello {:?} @ {:?}", player_id, position);
        net.send_message(player_id.0, ServerMessage::Hello(*player_id, position))
            .unwrap();

        for (other_player_position, other_player_direction, other_player_id) in query.iter() {
            if other_player_id == player_id {
                continue;
            }

            // Send new player position to all other players
            net.send_message(other_player_id.0, ServerMessage::Move {
                player_id: *player_id,
                direction,
                position,
                distance: 0,
            }).unwrap();

            // Send positions of all previously connected players to new player
            net.send_message(player_id.0, ServerMessage::Move {
                player_id: *other_player_id,
                direction: *other_player_direction,
                position: *other_player_position,
                distance: 0,
            }).unwrap();
        }
    }
}

fn broadcast_moves(
    mut commands: Commands,
    mut query: Query<(&Position, &Direction, &PlayerId)>,
    mut player_messages: EventReader<PlayerMessage>,
    mut net: ResMut<NetworkResource>,
) {
    for player_message in player_messages.iter() {
        if let PlayerMessage {
            player,
            client_message: ClientMessage::Move(direction, position),
        } = player_message
        {
            if let Ok((_current_position, current_direction, player_id)) = query.get_mut(*player) {
                let distance: u16;

                if current_direction != direction {
                    // Player is just turning
                    commands.entity(*player).insert(*direction);
                    distance = 0;
                } else {
                    // TODO: validate new position is adjacent to existing position
                    // TODO: collision detection
                    commands.entity(*player).insert(*position);
                    distance = 1;
                }

                broadcast(
                    &mut net,
                    ServerMessage::Move {
                        player_id: *player_id,
                        direction: *direction,
                        position: *position,
                        distance,
                    },
                );
            } else {
                log::warn!("Ignoring Move for player without direction/position");
            }
        }
    }
}

fn broadcast(net: &mut NetworkResource, message: ServerMessage) {
    log::debug!("Broadcast: {:?}", message);
    net.broadcast_message(message);
}

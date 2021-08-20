use std::time::Duration;

use bevy::{app::ScheduleRunnerSettings, prelude::*};
use bevy_networking_turbulence::NetworkResource;
use log::LevelFilter;
use network::{NetworkPlugin, PlayerMessage};
use simple_logger::SimpleLogger;
use woods_common::{ClientMessage, Direction, PlayerId, Position, ServerMessage};

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
        .run();
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
    log::debug!("BROADCAST {:?}", message);
    net.broadcast_message(message);
}

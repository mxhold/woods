use std::{collections::HashMap, net::SocketAddr};

use bevy::prelude::*;

use bevy_spicy_networking::{
    AppNetworkClientMessage, ClientNetworkEvent, NetworkClient, NetworkData, NetworkSettings,
};
use woods_common::{MoveUpdate, PlayerId, PlayerLeft, Welcome, SERVER_PORT};

use crate::{
    player::{insert_player, PlayerTextureAtlasHandle},
    Me, WalkEvent,
};

#[derive(Default)]
struct Players(pub HashMap<PlayerId, Entity>);

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugin(bevy_spicy_networking::ClientPlugin)
            .insert_resource(Players::default())
            .add_startup_system(setup_networking.system())
            .add_system(handle_network_events.system())
            .add_system(handle_welcome.system())
            .add_system(handle_move_updates.system())
            .add_system(handle_player_left.system());

        app.listen_for_client_message::<Welcome>();
        app.listen_for_client_message::<MoveUpdate>();
        app.listen_for_client_message::<PlayerLeft>();
    }
}

fn setup_networking(mut net: ResMut<NetworkClient>) {
    let ip_address = "127.0.0.1".parse().unwrap();
    let socket_address = SocketAddr::new(ip_address, SERVER_PORT);
    log::info!("Connecting to server at {:?}", socket_address);
    net.connect(socket_address, NetworkSettings::default());
}

fn handle_welcome(
    mut commands: Commands,
    mut players: ResMut<Players>,
    mut welcomes: EventReader<NetworkData<Welcome>>,
    me_query: Query<Entity, With<Me>>,
) {
    let me = me_query.single().unwrap();
    for network_data in welcomes.iter() {
        let Welcome(player_id, position) = **network_data;
        log::info!("[ME] {:?} @ {:?}", player_id, position);
        commands.entity(me).insert(player_id).insert(position);
        players.0.insert(player_id, me);
    }
}

fn handle_move_updates(
    mut commands: Commands,
    mut players: ResMut<Players>,
    mut moves: EventReader<NetworkData<MoveUpdate>>,
    me_query: Query<Entity, With<Me>>,
    mut walk_events: EventWriter<WalkEvent>,
    player_texture_atlas_handle: Res<PlayerTextureAtlasHandle>,
) {
    let me = me_query.single().unwrap();
    for network_data in moves.iter() {
        let MoveUpdate {
            player_id,
            direction,
            position,
            distance,
        } = **network_data;
        log::trace!(
            "{:?} @ {:?}, {:?} {:?}",
            player_id,
            position,
            direction,
            distance
        );

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
                let player = insert_player(
                    &mut commands,
                    player_texture_atlas_handle.clone(),
                    direction,
                    position,
                );
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

fn handle_network_events(mut network_events: EventReader<ClientNetworkEvent>) {
    for event in network_events.iter() {
        match event {
            ClientNetworkEvent::Connected => {
                log::info!("Connected.");
            }
            _ => {}
        }
    }
}

fn handle_player_left(
    mut player_left_events: EventReader<NetworkData<PlayerLeft>>,
    mut commands: Commands,
    mut players: ResMut<Players>,
) {
    for network_data in player_left_events.iter() {
        let PlayerLeft(player_id) = **network_data;
        if let Some(player) = players.0.remove(&player_id) {
            commands.entity(player).despawn();
            log::trace!("{:?} left.", player_id);
        }
    }
}

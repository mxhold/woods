use bevy::{
    input::{keyboard::KeyboardInput, ElementState},
    prelude::*,
};
use bevy_networking_turbulence::{
    ConnectionChannelsBuilder, NetworkEvent, NetworkResource, NetworkingPlugin,
};
use serde::{Deserialize, Serialize};
use std::{convert::TryInto, net::SocketAddr};

use direction::Direction;
use walk_animation::WalkAnimation;

mod direction;
mod walk_animation;

use woods_common::{CLIENT_STATE_MESSAGE_SETTINGS, SERVER_MESSAGE_SETTINGS, SERVER_PORT};

fn main() {
    App::build()
        .insert_resource(WindowDescriptor {
            title: "Woods".to_string(),
            width: 400.0,
            height: 300.0,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(NetworkingPlugin::default())
        .add_startup_system(setup_player.system())
        .add_startup_system(connect.system())
        .add_startup_system(network_setup.system())
        .add_system(keyboard_movement.system())
        .add_system(walk_animation.system())
        .add_system(handle_packets.system())
        .add_system_to_stage(CoreStage::PreUpdate, handle_messages_client.system())
        .run();
}

fn keyboard_movement(
    mut keyboard_input_events: EventReader<KeyboardInput>,
    mut query: Query<(&Player, &mut Transform, &mut Direction, &mut WalkAnimation)>,
) {
    for event in keyboard_input_events
        .iter()
        .filter(|e| e.state == ElementState::Pressed)
    {
        if let Some(key_code) = event.key_code {
            if let Ok(to_direction) = key_code.try_into() {
                for (_, transform, direction, walk_animation) in query.iter_mut() {
                    start_walking(to_direction, direction, walk_animation, transform);
                }
            }
        }
    }
}

fn start_walking(
    to_direction: Direction,
    mut direction: Mut<Direction>,
    mut walk_animation: Mut<WalkAnimation>,
    mut transform: Mut<Transform>,
) {
    if walk_animation.running() {
        return;
    }

    if to_direction == *direction {
        to_direction.translate(&mut transform.translation);
        *walk_animation = WalkAnimation::new();
    } else {
        // Don't move if just changing directions
        *direction = to_direction;
    }
}

fn walk_animation(
    time: Res<Time>,
    mut query: Query<(
        &mut TextureAtlasSprite,
        &Direction,
        &mut WalkAnimation,
        &mut Transform,
    )>,
) {
    for (mut sprite, direction, mut walk_animation, mut transform) in query.iter_mut() {
        sprite.index = walk_animation.sprite_index_offset() + direction.sprite_index_offset();

        if walk_animation.stage_finished(time.delta()) {
            direction.translate(&mut transform.translation);
        }
    }
}

struct Player;

fn setup_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let texture_handle = asset_server.load("player.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(19.0, 38.0), 24, 1);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: texture_atlas_handle,
            transform: Transform::from_xyz(0., 100., 0.),
            ..Default::default()
        })
        .insert(Player)
        .insert(Direction::South)
        .insert(WalkAnimation::default());
}

fn connect(mut net: ResMut<NetworkResource>) {
    let ip_address =
        bevy_networking_turbulence::find_my_ip_address().expect("can't find ip address");
    let socket_address = SocketAddr::new(ip_address, SERVER_PORT);
    println!("Starting client");
    net.connect(socket_address);
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum ClientMessage {
    Hello(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum ServerMessage {
    Welcome(String),
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

fn handle_messages_client(mut net: ResMut<NetworkResource>) {
    for (handle, connection) in net.connections.iter_mut() {
        let channels = connection.channels().unwrap();

        while let Some(server_message) = channels.recv::<ServerMessage>() {
            println!(
                "ServerMessage received on [{}]: {:?}",
                handle, server_message
            );
        }
    }
}

fn handle_packets(mut net: ResMut<NetworkResource>, mut network_events: EventReader<NetworkEvent>) {
    for event in network_events.iter() {
        match event {
            NetworkEvent::Connected(handle) => match net.connections.get_mut(handle) {
                Some(connection) => {
                    match connection.remote_address() {
                        Some(remote_address) => {
                            println!(
                                "Incoming connection on [{}] from [{}]",
                                handle, remote_address
                            );
                        }
                        None => {
                            println!("Connected on [{}]", handle);
                        }
                    }

                    println!("Sending Hello on [{}]", handle);
                    match net.send_message(*handle, ClientMessage::Hello("test".to_string())) {
                        Ok(msg) => match msg {
                            Some(msg) => {
                                println!("Unable to send Hello: {:?}", msg);
                            }
                            None => {}
                        },
                        Err(err) => {
                            println!("Unable to send Hello: {:?}", err);
                        }
                    };
                }
                None => panic!("Got packet for non-existing connection [{}]", handle),
            },
            _ => {}
        }
    }
}

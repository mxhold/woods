use bevy::{
    input::{keyboard::KeyboardInput, ElementState},
    prelude::*,
    render::camera::WindowOrigin,
};
use bevy_networking_turbulence::{
    ConnectionChannelsBuilder, NetworkEvent, NetworkResource, NetworkingPlugin,
};
use log::LevelFilter;
use simple_logger::SimpleLogger;
use std::{convert::TryInto, net::SocketAddr};

use direction::Direction;
use walk_animation::WalkAnimation;

mod direction;
mod walk_animation;

use woods_common::{
    ClientMessage, PlayerId, Position, ServerMessage, CLIENT_STATE_MESSAGE_SETTINGS,
    SERVER_MESSAGE_SETTINGS, SERVER_PORT,
};

fn main() {
    SimpleLogger::new()
        .with_level(LevelFilter::Off)
        .with_module_level("woods_client", LevelFilter::Trace)
        .init()
        .unwrap();

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
    mut query: Query<(&Player, &PlayerId, &mut Direction, &mut WalkAnimation, &mut Position)>,
    mut net: ResMut<NetworkResource>,
) {
    for event in keyboard_input_events
        .iter()
        .filter(|e| e.state == ElementState::Pressed)
    {
        if let Some(key_code) = event.key_code {
            if let Ok(to_direction) = key_code.try_into() {
                for (_, player_id, direction, walk_animation, mut position) in query.iter_mut() {
                    start_walking(to_direction, direction, walk_animation, &mut position);
                    log::info!("PlayerID={:?}", player_id);

                    send_move(*position, &mut net)
                }
            }
        }
    }
}

fn send_move(position: Position, net: &mut ResMut<NetworkResource>) {
    log::info!("Broadcasting move to {:?}", position);
    net.broadcast_message(ClientMessage::Move(position));
}

fn start_walking(
    to_direction: Direction,
    mut direction: Mut<Direction>,
    mut walk_animation: Mut<WalkAnimation>,
    mut position: &mut Mut<Position>,
) {
    if walk_animation.running() {
        return;
    }

    if to_direction == *direction {
        to_direction.translate_position(&mut position);
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
        &Position,
        &mut Transform,
    )>,
) {
    for (mut sprite, direction, mut walk_animation, position, mut transform) in query.iter_mut() {
        walk_animation.tick(time.delta());

        sprite.index = walk_animation.sprite_index_offset() + direction.sprite_index_offset();

        transform.translation = walk_animation.translate(position, direction);
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
    let mut camera = OrthographicCameraBundle::new_2d();
    camera.orthographic_projection.window_origin = WindowOrigin::BottomLeft;
    commands.spawn_bundle(camera);
    commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: texture_atlas_handle,
            transform: Transform::from_xyz(10., 20., 0.),
            ..Default::default()
        })
        .insert(Direction::South)
        .insert(Player)
        .insert(Position { x: 3, y: 4 })
        .insert(WalkAnimation::default());
}

fn connect(mut net: ResMut<NetworkResource>) {
    let ip_address =
        bevy_networking_turbulence::find_my_ip_address().expect("can't find ip address");
    let socket_address = SocketAddr::new(ip_address, SERVER_PORT);
    log::info!("Starting client");
    net.connect(socket_address);
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

fn handle_messages_client(
    mut net: ResMut<NetworkResource>,
    query: Query<Entity, With<Player>>,
    mut commands: Commands,
) {
    for (handle, connection) in net.connections.iter_mut() {
        let channels = connection.channels().unwrap();

        while let Some(server_message) = channels.recv::<ServerMessage>() {
            log::debug!(
                "ServerMessage received on [{}]: {:?}",
                handle,
                server_message
            );

            match server_message {
                ServerMessage::PlayerId(player_id) => {
                    let player = query.single().unwrap();
                    commands.entity(player).insert(PlayerId(player_id.0));
                }
                _ => {}
            }
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
                            log::info!(
                                "Incoming connection on [{}] from [{}]",
                                handle,
                                remote_address
                            );
                        }
                        None => {
                            log::info!("Connected on [{}]", handle);
                        }
                    };

                    net.broadcast_message(ClientMessage::Hello);
                }
                None => panic!("Got packet for non-existing connection [{}]", handle),
            },
            _ => {}
        }
    }
}

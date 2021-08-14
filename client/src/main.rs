use bevy::{
    input::{keyboard::KeyboardInput, ElementState},
    prelude::*,
};
use std::convert::TryInto;

use direction::Direction;
use walk_animation::WalkAnimation;

mod direction;
mod walk_animation;

fn main() {
    App::build()
        .insert_resource(WindowDescriptor {
            title: "Woods".to_string(),
            width: 400.0,
            height: 300.0,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup.system())
        .add_system(keyboard_movement.system())
        .add_system(walk_animation.system())
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

fn setup(
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

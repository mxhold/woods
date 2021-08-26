use bevy::prelude::*;
use woods_common::Position;

use crate::walk_animation::WalkAnimation;
use crate::{Collide, Direction, TransformOffset};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<PlayerTextureAtlasHandle>()
            .add_startup_system(load_sprite.system().label("load_sprite"))
            .add_startup_system(setup_me.system().after("load_sprite"));
    }
}
pub struct Me;

#[derive(Bundle)]
struct PlayerBundle {
    #[bundle]
    sprite_sheet: SpriteSheetBundle,
    direction: Direction,
    walk_animation: WalkAnimation,
    collide: Collide,
    transform_offset: TransformOffset,
}

impl Default for PlayerBundle {
    fn default() -> Self {
        Self {
            sprite_sheet: SpriteSheetBundle::default(),
            direction: Direction::South,
            walk_animation: Default::default(),
            collide: Default::default(),
            transform_offset: TransformOffset(Transform::from_translation(Vec3::new(19.0 / 2.0, 38.0 / 2.0, 0.0))),
        }
    }
}

#[derive(Clone, Default)]
pub struct PlayerTextureAtlasHandle(Handle<TextureAtlas>);

fn load_sprite(
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut player_texture_atlas_handle: ResMut<PlayerTextureAtlasHandle>,
) {
    let texture_handle = asset_server.load("player.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(19.0, 38.0), 24, 1);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    *player_texture_atlas_handle = PlayerTextureAtlasHandle(texture_atlas_handle);
}

fn setup_me(mut commands: Commands, player_texture_atlas_handle: Res<PlayerTextureAtlasHandle>) {
    commands
        .spawn_bundle(PlayerBundle {
            sprite_sheet: SpriteSheetBundle {
                texture_atlas: player_texture_atlas_handle.clone().0,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Me);
}

pub fn insert_player(
    commands: &mut Commands,
    player_texture_atlas_handle: PlayerTextureAtlasHandle,
    direction: Direction,
    position: Position,
) -> Entity {
    commands
        .spawn()
        .insert_bundle(PlayerBundle {
            sprite_sheet: SpriteSheetBundle {
                texture_atlas: player_texture_atlas_handle.0,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(direction)
        .insert(position)
        .id()
}

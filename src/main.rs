use bevy::prelude::*;

fn main() {
    App::build()
    .insert_resource(WindowDescriptor {
        title: "Woods".to_string(),
        width: 400.,
        height: 300.,
        ..Default::default()
    })
    .add_plugins(DefaultPlugins)
    .add_startup_system(setup.system())
    .add_system(keyboard_movement.system())
    .add_system(sprite_system.system())
    // .add_system(animate_sprite_system.system())
    .run();
}

// fn animate_sprite_system(
//     time: Res<Time>,
//     texture_atlases: Res<Assets<TextureAtlas>>,
//     mut query: Query<(&mut Timer, &mut TextureAtlasSprite, &Handle<TextureAtlas>)>,
// ) {
//     for (mut timer, mut sprite, texture_atlas_handle) in query.iter_mut() {
//         timer.tick(time.delta());
//         if timer.finished() {
//             let texture_atlas = texture_atlases.get(texture_atlas_handle).unwrap();
//             sprite.index = ((sprite.index as usize + 1) % texture_atlas.textures.len()) as u32;
//         }
//     }
// }

fn sprite_system(
    mut query: Query<(&mut TextureAtlasSprite, &Direction)>,
) {
    for (mut sprite, direction) in query.iter_mut() {
        match direction {
            Direction::North => {
                sprite.index = 1;
            }
            Direction::South => {
                sprite.index = 7
            }
            Direction::East => {
                sprite.index = 13;
            }
            Direction::West => {
                sprite.index = 19;
            }
        }
    }
}

fn keyboard_movement(keyboard_input: Res<Input<KeyCode>>, mut query: Query<(&Player, &mut Transform, &mut Direction)>) {
    for (_, mut transform, mut direction) in query.iter_mut() {
        if keyboard_input.just_pressed(KeyCode::Right) {
            transform.translation.x += 20.;
            *direction = Direction::East;
        }
        if keyboard_input.just_pressed(KeyCode::Left) {
            transform.translation.x -= 20.;
            *direction = Direction::West;
        }
        if keyboard_input.just_pressed(KeyCode::Up) {
            transform.translation.y += 20.;
            *direction = Direction::North;
        }
        if keyboard_input.just_pressed(KeyCode::Down) {
            transform.translation.y -= 20.;
            *direction = Direction::South;
        }
    }
}

struct Player;

enum Direction {
    North,
    South,
    East,
    West
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut windows: ResMut<Windows>,
) {
    let texture_handle = asset_server.load("player.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(19.0, 38.0), 24, 1);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    let window = windows.get_primary_mut().unwrap();
    window.set_scale_factor_override(Some(window.scale_factor() * 2.));
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: texture_atlas_handle,
            transform: Transform::from_xyz(0., 100., 0.),
            ..Default::default()
        })
        .insert(Player)
        .insert(Direction::South)
        .insert(Timer::from_seconds(0.1, true));
}
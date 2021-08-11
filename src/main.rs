use std::time::Duration;
use bevy::prelude::*;

const TILE_SIZE: f32 = 20.0;
const STEP_DIST: f32 = TILE_SIZE / 3.0;

const WALK_DURATION: Duration = Duration::from_millis(300/3);

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
    .run();
}

fn sprite_system(
    time: Res<Time>,
    mut query: Query<(&mut TextureAtlasSprite, &Direction, &mut WalkAnimation, &mut Transform)>,
) {
    for (mut sprite, direction, mut walk_animation, mut transform) in query.iter_mut() {
        sprite.index = walk_animation.stage.sprite_index() + direction.sprite_offset(6);

        if walk_animation.stage == WalkStage::Stop {
            continue;
        }

        walk_animation.timer.tick(time.delta());

        if walk_animation.timer.finished() {
            match direction {
                Direction::East => {
                    transform.translation.x += STEP_DIST;
                }
                Direction::West => {
                    transform.translation.x -= STEP_DIST;
                }
                Direction::North => {
                    transform.translation.y += STEP_DIST;
                }
                Direction::South => {
                    transform.translation.y -= STEP_DIST;
                }
            }
            walk_animation.next();
        }
    }
}

fn keyboard_movement(keyboard_input: Res<Input<KeyCode>>, mut query: Query<(&Player, &mut Transform, &mut Direction, &mut WalkAnimation)>) {
    for (_, mut transform, mut direction, mut walk_animation) in query.iter_mut() {
        if keyboard_input.just_pressed(KeyCode::Right) {
            transform.translation.x += STEP_DIST;
            *direction = Direction::East;
            *walk_animation = WalkAnimation::new(WalkStage::Step1)
        }
        if keyboard_input.just_pressed(KeyCode::Left) {
            transform.translation.x -= STEP_DIST;
            *direction = Direction::West;
            *walk_animation = WalkAnimation::new(WalkStage::Step1)
        }
        if keyboard_input.just_pressed(KeyCode::Up) {
            transform.translation.y += STEP_DIST;
            *direction = Direction::North;
            *walk_animation = WalkAnimation::new(WalkStage::Step1)
        }
        if keyboard_input.just_pressed(KeyCode::Down) {
            transform.translation.y -= STEP_DIST;
            *direction = Direction::South;
            *walk_animation = WalkAnimation::new(WalkStage::Step1)
        }
    }
}

struct Player;

#[derive(Eq, PartialEq, Copy, Clone)]
enum Direction {
    North,
    South,
    East,
    West
}

impl Direction {
    pub fn sprite_offset(&self, frames_per_direction: u32) -> u32 {
        match self {
            Direction::North => frames_per_direction * 0,
            Direction::South => frames_per_direction * 1,
            Direction::East => frames_per_direction * 2,
            Direction::West => frames_per_direction * 3,
        }
    }
}

#[derive(PartialEq, Eq, Copy, Clone)]
enum WalkStage {
    Stop,
    Step1,
    Pause,
    Step2
}

impl WalkStage {
    pub fn next(&self) -> Self {
        match self {
            WalkStage::Step1 => WalkStage::Pause,
            WalkStage::Pause => WalkStage::Step2,
            WalkStage::Step2 => WalkStage::Stop,
            WalkStage::Stop => WalkStage::Stop,
        }
    }


    pub fn sprite_index(&self) -> u32 {
        match self {
            WalkStage::Step1 => 0,
            WalkStage::Pause => 1,
            WalkStage::Step2 => 2,
            WalkStage::Stop => 1,
        }
    }
}

struct WalkAnimation {
    pub stage: WalkStage,
    pub timer: Timer
}

impl WalkAnimation {
    pub fn next(&mut self) {
        self.stage = self.stage.next();
        if self.stage != WalkStage::Stop {
            self.timer = Timer::new(WALK_DURATION, false)
        }
    }

    pub fn new(stage: WalkStage) -> Self {
        Self {
            stage: stage,
            timer: Timer::new(WALK_DURATION, false)
        }
    }
}

impl Default for WalkAnimation {
    fn default() -> Self {
        WalkAnimation::new(WalkStage::Stop)
    }
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
        .insert(WalkAnimation::default());
    }
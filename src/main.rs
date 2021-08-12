use bevy::{
    input::{keyboard::KeyboardInput, ElementState},
    prelude::*,
};
use std::{
    convert::{TryFrom, TryInto},
    time::Duration,
};

const TILE_SIZE: f32 = 20.0;
const STEP_DIST: f32 = TILE_SIZE / 3.0;

const WALK_DURATION: Duration = Duration::from_millis(300 / 3);

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
        sprite.index = walk_animation.stage.sprite_index() + direction.sprite_offset(6);

        if walk_animation.stage == WalkStage::Stop {
            continue;
        }

        walk_animation.timer.tick(time.delta());

        if walk_animation.timer.finished() {
            direction.translate(&mut transform.translation);
            walk_animation.next();
        }
    }
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
    if walk_animation.stage != WalkStage::Stop {
        return;
    }

    if to_direction == *direction {
        to_direction.translate(&mut transform.translation);
        *walk_animation = WalkAnimation::new(WalkStage::Step1);
    } else {
        // Don't move if just changing directions
        *direction = to_direction;
    }
}

struct Player;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
enum Direction {
    North,
    South,
    East,
    West,
}

impl TryFrom<KeyCode> for Direction {
    type Error = &'static str;

    fn try_from(key_code: KeyCode) -> Result<Self, Self::Error> {
        match key_code {
            KeyCode::Right => Ok(Direction::East),
            KeyCode::Left => Ok(Direction::West),
            KeyCode::Up => Ok(Direction::North),
            KeyCode::Down => Ok(Direction::South),
            _ => Err("Not a direction key"),
        }
    }
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

    pub fn translate(&self, translation: &mut Vec3) {
        match self {
            Direction::East => {
                translation.x += STEP_DIST;
            }
            Direction::West => {
                translation.x -= STEP_DIST;
            }
            Direction::North => {
                translation.y += STEP_DIST;
            }
            Direction::South => {
                translation.y -= STEP_DIST;
            }
        }
    }
}

#[derive(PartialEq, Eq, Copy, Clone)]
enum WalkStage {
    Stop,
    Step1,
    Pause,
    Step2,
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
    stage: WalkStage,
    timer: Timer,
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
            timer: Timer::new(WALK_DURATION, false),
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

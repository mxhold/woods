use bevy::prelude::*;

use std::convert::TryFrom;

pub const TILE_SIZE: f32 = 20.0;
pub const STEP_DIST: f32 = TILE_SIZE / 3.0;
const FRAMES_PER_DIRECTION: u32 = 6;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Direction {
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
    pub fn sprite_index_offset(&self) -> u32 {
        match self {
            Direction::North => FRAMES_PER_DIRECTION * 0,
            Direction::South => FRAMES_PER_DIRECTION * 1,
            Direction::East => FRAMES_PER_DIRECTION * 2,
            Direction::West => FRAMES_PER_DIRECTION * 3,
        }
    }

    pub fn translation(&self) -> Vec2 {
        match self {
            Direction::East => {
                Vec2::new(1.0, 0.0)
            }
            Direction::West => {
                Vec2::new(-1.0, 0.0)
            }
            Direction::North => {
                Vec2::new(0.0, 1.0)
            }
            Direction::South => {
                Vec2::new(0.0, -1.0)
            }
        }
    }
}

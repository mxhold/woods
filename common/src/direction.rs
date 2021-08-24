use bevy::prelude::*;

use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Copy, Clone)]
pub enum Direction {
    North,
    South,
    East,
    West,
}

impl Default for Direction {
    fn default() -> Self {
        Direction::South
    }
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
    pub fn translation(&self) -> Vec2 {
        match self {
            Direction::East => Vec2::new(1.0, 0.0),
            Direction::West => Vec2::new(-1.0, 0.0),
            Direction::North => Vec2::new(0.0, 1.0),
            Direction::South => Vec2::new(0.0, -1.0),
        }
    }
}

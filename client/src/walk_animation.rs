use bevy::prelude::*;
use woods_common::Position;
use std::time::Duration;

use crate::direction::{Direction, TILE_SIZE, STEP_DIST};

const WALK_DURATION: Duration = Duration::from_millis(300 / 3);

#[derive(PartialEq, Eq, Copy, Clone)]
enum WalkStage {
    Stop,
    Step1,
    Pause,
    Step2,
}

impl WalkStage {
    fn next(&self) -> Self {
        match self {
            WalkStage::Step1 => WalkStage::Pause,
            WalkStage::Pause => WalkStage::Step2,
            WalkStage::Step2 => WalkStage::Stop,
            WalkStage::Stop => WalkStage::Stop,
        }
    }

    fn sprite_index_offset(&self) -> u32 {
        match self {
            WalkStage::Step1 => 0,
            WalkStage::Pause => 1,
            WalkStage::Step2 => 2,
            WalkStage::Stop => 1,
        }
    }

    fn step_offset(&self) -> f32 {
        match self {
            WalkStage::Step1 => 3.0 * STEP_DIST,
            WalkStage::Pause => 2.0 * STEP_DIST,
            WalkStage::Step2 => 1.0 * STEP_DIST,
            WalkStage::Stop => 0.0 * STEP_DIST,
        }
    }
}

pub struct WalkAnimation {
    stage: WalkStage,
    timer: Option<Timer>,
}

impl WalkAnimation {
    pub fn running(&self) -> bool {
        self.timer.is_some()
    }

    pub fn new() -> Self {
        Self {
            stage: WalkStage::Step1,
            timer: Some(Timer::new(WALK_DURATION, false)),
        }
    }

    pub fn sprite_index_offset(&self) -> u32 {
        self.stage.sprite_index_offset()
    }

    pub fn tick(&mut self, duration: Duration) {
        if let Some(ref mut timer) = self.timer {
            timer.tick(duration);
            if timer.finished() {
                self.next();
            }
        }
    }

    fn next(&mut self) {
        self.stage = self.stage.next();
        if self.stage == WalkStage::Stop {
            self.timer = None;
        } else {
            self.timer = Some(Timer::new(WALK_DURATION, false))
        }
    }

    pub fn translate(&self, position: &Position, direction: &Direction) -> Vec3 {
        let mut translation = Vec3::new(
            (position.x as f32 * TILE_SIZE).into(),
            (position.y as f32 * TILE_SIZE).into(),
            0.0,
        );
        match direction {
            Direction::East => {
                translation.x -= self.stage.step_offset();
            }
            Direction::West => {
                translation.x += self.stage.step_offset();
            }
            Direction::North => {
                translation.y -= self.stage.step_offset();
            }
            Direction::South => {
                translation.y += self.stage.step_offset();
            }
        }

        translation
    }
}

impl Default for WalkAnimation {
    fn default() -> Self {
        Self {
            stage: WalkStage::Stop,
            timer: None,
        }
    }
}

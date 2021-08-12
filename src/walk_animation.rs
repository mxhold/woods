use bevy::prelude::*;
use std::time::Duration;

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

    pub fn stage_finished(&mut self, duration: Duration) -> bool {
        if let Some(ref mut timer) = self.timer {
            timer.tick(duration);
            if timer.finished() {
                self.next();
                return true;
            } else {
                return false;
            }
        } else {
            return false;
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
}

impl Default for WalkAnimation {
    fn default() -> Self {
        Self {
            stage: WalkStage::Stop,
            timer: None,
        }
    }
}

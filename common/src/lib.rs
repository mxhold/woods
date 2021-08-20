pub mod direction;

use bevy::math::Vec2;
pub use direction::Direction;

use std::time::Duration;
use serde::{Deserialize, Serialize};

use bevy_networking_turbulence::{MessageChannelMode, MessageChannelSettings, ReliableChannelSettings};

pub const SERVER_PORT: u16 = 14192;

pub const CLIENT_MESSAGE_SETTINGS: MessageChannelSettings = MessageChannelSettings {
  channel: 0,
  channel_mode: MessageChannelMode::Reliable {
      reliability_settings: ReliableChannelSettings {
          bandwidth: 4096,
          recv_window_size: 1024,
          send_window_size: 1024,
          burst_bandwidth: 1024,
          init_send: 512,
          wakeup_time: Duration::from_millis(100),
          initial_rtt: Duration::from_millis(200),
          max_rtt: Duration::from_secs(2),
          rtt_update_factor: 0.1,
          rtt_resend_factor: 1.5,
      },
      max_message_len: 1024,
  },
  message_buffer_size: 8,
  packet_buffer_size: 8,
};

pub const SERVER_MESSAGE_SETTINGS: MessageChannelSettings = MessageChannelSettings {
  channel: 1,
  channel_mode: MessageChannelMode::Unreliable,
  message_buffer_size: 8,
  packet_buffer_size: 8,
};

#[derive(Hash, Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq)]
pub struct PlayerId(pub u32);

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Eq, PartialEq)]
pub struct Position {
  pub x: u16,
  pub y: u16
}

impl From<Position> for Vec2 {
    fn from(position: Position) -> Self {
        Self::new(
          position.x.into(),
          position.y.into()
        )
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ClientMessage {
    Hello,
    Move(Direction, Position),
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum ServerMessage {
    Hello(PlayerId, Position),
    Move {
      player_id: PlayerId,
      direction: Direction,
      position: Position,
      distance: u16,
    },
}
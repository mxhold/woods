use std::time::Duration;
use serde::{Deserialize, Serialize};

use bevy_networking_turbulence::{MessageChannelMode, MessageChannelSettings, ReliableChannelSettings};

pub const SERVER_PORT: u16 = 14192;

pub const CLIENT_STATE_MESSAGE_SETTINGS: MessageChannelSettings = MessageChannelSettings {
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

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Direction {
    North,
    South,
    East,
    West,
}

#[derive(Serialize, Deserialize, Debug, Clone,  Eq, PartialEq)]
pub struct PlayerId(pub u32);

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Eq, PartialEq)]
pub struct Position {
  pub x: u16,
  pub y: u16
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ClientMessage {
    Move(Position),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ServerMessage {
    PlayerId(PlayerId),
    Position(PlayerId, Position),
}
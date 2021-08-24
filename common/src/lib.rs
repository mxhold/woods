pub mod direction;

use bevy::math::Vec2;

use bevy_spicy_networking::{ClientMessage, NetworkMessage, ServerMessage};
pub use direction::Direction;

use serde::{Deserialize, Serialize};

pub const SERVER_PORT: u16 = 14192;

#[derive(Hash, Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq)]
pub struct PlayerId(pub u32);

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Eq, PartialEq)]
pub struct Position {
    pub x: u16,
    pub y: u16,
}

impl From<Position> for Vec2 {
    fn from(position: Position) -> Self {
        Self::new(position.x.into(), position.y.into())
    }
}

// Client -> Server messages

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MoveInput(pub Direction, pub Position);

#[typetag::serde]
impl NetworkMessage for MoveInput {}

impl ServerMessage for MoveInput {
    const NAME: &'static str = "woods:MoveInput";
}

// Server -> Client messages

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Welcome(pub PlayerId, pub Position);

#[typetag::serde]
impl NetworkMessage for Welcome {}

impl ClientMessage for Welcome {
    const NAME: &'static str = "woods:Welcome";
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MoveUpdate {
    pub player_id: PlayerId,
    pub direction: Direction,
    pub position: Position,
    pub distance: u16,
}

#[typetag::serde]
impl NetworkMessage for MoveUpdate {}

impl ClientMessage for MoveUpdate {
    const NAME: &'static str = "woods:MoveInfo";
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlayerLeft(pub PlayerId);

#[typetag::serde]
impl NetworkMessage for PlayerLeft {}

impl ClientMessage for PlayerLeft {
    const NAME: &'static str = "woods:PlayerLeft";
}

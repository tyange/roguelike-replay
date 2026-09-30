use crate::entity::EntityId;
use crate::geom::Point;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position(pub Point);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Glyph {
    pub ch: char,
    pub color: Rgb,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Health {
    pub hp: i32,
    pub max: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Name(pub String);

/// 지나갈 수 없게 막는 표시
#[derive(Debug, Clone, Copy)]
pub struct BlocksTile;

#[derive(Debug, Clone, Copy)]
pub struct Player;

#[derive(Debug, Clone, Copy)]
pub struct Monster;

/// 쫓아가는 대상. 참조가 아니라 ID
#[derive(Debug, Clone, Copy)]
pub struct Target(pub EntityId);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Combat {
    pub power: i32,
    pub defense: i32,
}

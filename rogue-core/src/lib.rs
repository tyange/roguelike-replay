//! 로그라이크의 규칙과 데이터. 화면과 키보드는 모른다(macroquad 에 의존하지 않는다).
//!
//! 입구는 [`Game`] 하나다. 실행 파일은 [`Input`] 을 만들어 [`Game::update`] 에 넣고,
//! [`Game::state`], [`Game::world`], [`Game::map`] 을 읽어 화면에 그린다.
//! 창이 없는 `rogue-sim` 도 같은 입구로 게임을 돌린다.

mod action;
mod ai;
mod combat;
mod comps;
mod defs;
mod dungeon;
mod entity;
mod error;
mod event;
mod floor;
mod fov;
mod game;
mod geom;
mod map;
mod path;
mod rng;
mod state;
mod store;
mod world;

pub use action::Action;
pub use comps::{
    Ai, BlocksTile, Combat, Effect, Glyph, Health, InBackpack, Item, Monster, Name, Player,
    Position, Rgb, Viewshed,
};
pub use defs::{Defs, ItemDef, MonsterDef};
pub use entity::{Entities, EntityId};
pub use error::DataError;
pub use event::Event;
pub use game::Game;
pub use geom::{Dir, Point};
pub use map::{Map, Tile};
pub use path::astar;
pub use rng::Rng;
pub use state::{Input, RunState};
pub use store::Store;
pub use world::World;

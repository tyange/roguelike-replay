use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::entity::EntityId;
use crate::geom::Point;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position(pub Point);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Glyph {
    pub ch: char,
    pub color: Rgb,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Health {
    pub hp: i32,
    pub max: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Name(pub String);

/// 지나갈 수 없게 막는 표시
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BlocksTile;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Player;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Monster;

/// 몬스터의 행동 방식. 변형마다 필요한 데이터를 담는다
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Ai {
    /// 플레이어를 쫓아와서 때린다
    Chaser,
    /// 체력이 `flee_below` 미만이면 도망간다
    Coward { flee_below: i32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Combat {
    pub power: i32,
    pub defense: i32,
}

/// 시야. `visible`은 다시 계산할 수 있는 값이다
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Viewshed {
    pub range: i32,
    /// 다시 계산할 수 있으므로 저장하지 않는다
    #[serde(skip)]
    pub visible: HashSet<Point>,
    /// 저장하지 않는다. 불러온 쪽이 true 로 만들어 다시 계산하게 한다
    #[serde(skip)]
    pub dirty: bool,
}

impl Viewshed {
    pub fn new(range: i32) -> Viewshed {
        Viewshed {
            range,
            visible: HashSet::new(),
            dirty: true,
        }
    }
}

/// 아이템을 쓰면 일어나는 일
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Effect {
    Heal(i32),
    AreaDamage { amount: i32, radius: i32 },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Item(pub Effect);

/// 가방에 들어 있다. 이 컴포넌트가 있으면 Position은 없다
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct InBackpack {
    pub owner: EntityId,
}

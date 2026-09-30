use serde::{Deserialize, Serialize};

use crate::action::Action;
use crate::entity::EntityId;
use crate::geom::{Dir, Point};

/// 게임이 지금 어느 단계에 있는가. 불가능한 조합은 타입으로 표현할 수 없다
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunState {
    AwaitingInput,
    MonsterTurn,
    ShowInventory,
    /// 어떤 아이템을 어디에 쓰려는지. 참조가 아니라 ID와 좌표라서 그대로 복사할 수 있다
    Targeting {
        item: EntityId,
        cursor: Point,
    },
    /// 다음 층을 다른 스레드가 만드는 중. 입력도 몬스터 행동도 없다
    Generating,
    GameOver {
        cause: String,
    },
}

/// 플레이어가 core에 넣을 수 있는 입력. 키보드 같은 장치와는 무관하다
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Input {
    Act(Action),
    Restart,
    OpenInventory,
    Cancel,
    /// 인벤토리에서 n번째 아이템 고르기
    Choose(usize),
    MoveCursor(Dir),
    Confirm,
}

use crate::entity::EntityId;

/// "무슨 일이 일어나야 한다"를 적어 두는 기록. 적용은 모든 행동이 끝난 뒤 한 번에 한다
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    Damage {
        target: EntityId,
        amount: i32,
        source: EntityId,
    },
    Heal {
        target: EntityId,
        amount: i32,
    },
    Consume {
        item: EntityId,
    },
}

use crate::action::Action;

/// 게임이 지금 어느 단계에 있는가. 불가능한 조합은 타입으로 표현할 수 없다
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunState {
    AwaitingInput,
    MonsterTurn,
    GameOver { cause: String },
}

/// 플레이어가 core에 넣을 수 있는 입력. 키보드 같은 장치와는 무관하다
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Input {
    Act(Action),
    Restart,
}

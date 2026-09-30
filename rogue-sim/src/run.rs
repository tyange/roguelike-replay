use std::sync::Arc;

use rogue_core::{Defs, Game, RunState};

use crate::bot;

const MAX_TURNS: u32 = 2000;
/// 입력 없이 도는 횟수까지 포함한 안전장치. 봇이 제자리에서 맴돌아도 끝나게 한다
const MAX_STEPS: u32 = 50_000;

#[derive(Debug, Clone, PartialEq, Eq)]
/// 한 판이 끝난 이유
pub enum Outcome {
    Died(String),
    TurnLimit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// 한 판의 결과
pub struct RunResult {
    pub depth: u32,
    pub turns: u32,
    pub kills: u32,
    pub outcome: Outcome,
}

/// 시드 하나로 한 판을 끝까지(죽거나 상한에 닿을 때까지) 돌린다
pub fn play_one(seed: u64, defs: &Arc<Defs>) -> RunResult {
    let mut game = Game::new(seed, Arc::clone(defs));
    let mut steps = 0;
    let outcome = loop {
        if let RunState::GameOver { cause } = game.state() {
            break Outcome::Died(cause.clone()); // game 을 계속 쓰므로 이름은 복사한다
        }
        steps += 1;
        if game.turns() >= MAX_TURNS || steps >= MAX_STEPS {
            break Outcome::TurnLimit;
        }
        let input = bot::choose(&game);
        game.update(input);
        if game.state() == &RunState::Generating {
            std::thread::yield_now(); // 층 생성 스레드가 일할 틈을 준다
        }
    };
    RunResult {
        depth: game.depth(),
        turns: game.turns(),
        kills: game.kills(),
        outcome,
    }
}

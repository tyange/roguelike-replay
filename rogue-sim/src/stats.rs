use std::collections::BTreeMap;
use std::fmt;

use crate::run::{Outcome, RunResult};

/// 여러 판의 통계. 합계와 최댓값만 가져서, 어떤 순서로 더하고 합쳐도 같은 값이 된다
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Stats {
    pub games: u64,
    pub total_depth: u64,
    pub max_depth: u32,
    pub total_turns: u64,
    pub total_kills: u64,
    pub turn_limit: u64,
    /// 사망 원인별 횟수. 출력 순서가 일정하도록 BTreeMap 을 쓴다
    pub deaths: BTreeMap<String, u64>,
}

impl Stats {
    pub fn add(&mut self, r: &RunResult) {
        self.games += 1;
        self.total_depth += u64::from(r.depth);
        self.max_depth = self.max_depth.max(r.depth);
        self.total_turns += u64::from(r.turns);
        self.total_kills += u64::from(r.kills);
        match &r.outcome {
            Outcome::Died(cause) => *self.deaths.entry(cause.clone()).or_default() += 1,
            Outcome::TurnLimit => self.turn_limit += 1,
        }
    }

    /// 두 통계를 하나로. map-reduce 의 reduce 단계
    pub fn merge(mut self, other: Stats) -> Stats {
        self.games += other.games;
        self.total_depth += other.total_depth;
        self.max_depth = self.max_depth.max(other.max_depth);
        self.total_turns += other.total_turns;
        self.total_kills += other.total_kills;
        self.turn_limit += other.turn_limit;
        for (cause, n) in other.deaths {
            *self.deaths.entry(cause).or_default() += n;
        }
        self
    }
}

/// 평균 표시용. 판 수가 0 이면 0
fn average(total: u64, games: u64) -> f64 {
    if games == 0 {
        return 0.0;
    }
    // 표시용 평균이라 아주 큰 수에서 정밀도가 조금 줄어도 상관없다
    total as f64 / games as f64
}

impl fmt::Display for Stats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "games            {}", self.games)?;
        writeln!(
            f,
            "avg depth        {:.2}",
            average(self.total_depth, self.games)
        )?;
        writeln!(f, "max depth        {}", self.max_depth)?;
        writeln!(
            f,
            "avg turns        {:.1}",
            average(self.total_turns, self.games)
        )?;
        writeln!(
            f,
            "avg kills        {:.2}",
            average(self.total_kills, self.games)
        )?;
        writeln!(f, "hit turn limit   {}", self.turn_limit)?;
        for (cause, n) in &self.deaths {
            writeln!(f, "killed by {cause:<9}{n}")?;
        }
        Ok(())
    }
}

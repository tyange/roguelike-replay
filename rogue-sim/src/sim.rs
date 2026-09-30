use std::sync::{Arc, Mutex};
use std::thread;

use rayon::prelude::*;
use rogue_core::Defs;

use crate::run::play_one;
use crate::stats::Stats;

/// 시드 0 부터 차례로 한 판씩
pub fn sequential(games: u64, defs: &Arc<Defs>) -> Stats {
    let mut stats = Stats::default();
    for seed in 0..games {
        stats.add(&play_one(seed, defs));
    }
    stats
}

/// scoped thread + 공유 통계 하나를 Mutex 로 보호
pub fn scoped_mutex(games: u64, defs: &Arc<Defs>, threads: u64) -> Stats {
    let shared = Arc::new(Mutex::new(Stats::default()));
    thread::scope(|scope| {
        for t in 0..threads {
            let shared = Arc::clone(&shared);
            scope.spawn(move || {
                for seed in (t..games).step_by(threads as usize) {
                    let result = play_one(seed, defs); // 잠그기 전에 계산한다
                    if let Ok(mut stats) = shared.lock() {
                        stats.add(&result);
                    }
                }
            });
        }
    });
    match Arc::try_unwrap(shared) {
        Ok(mutex) => mutex.into_inner().unwrap_or_default(),
        Err(_) => Stats::default(), // scope 가 끝났으니 다른 소유자는 없다
    }
}

/// 각 스레드가 자기 통계를 만들어 돌려주고, 마지막에 합친다 (map-reduce)
pub fn scoped_map_reduce(games: u64, defs: &Arc<Defs>, threads: u64) -> Stats {
    thread::scope(|scope| {
        let handles: Vec<_> = (0..threads)
            .map(|t| {
                scope.spawn(move || {
                    let mut stats = Stats::default();
                    for seed in (t..games).step_by(threads as usize) {
                        stats.add(&play_one(seed, defs));
                    }
                    stats
                })
            })
            .collect();
        handles
            .into_iter()
            .filter_map(|h| h.join().ok())
            .fold(Stats::default(), Stats::merge)
    })
}

/// rayon. 순차 버전과 거의 같은 모양이다
pub fn parallel(games: u64, defs: &Arc<Defs>) -> Stats {
    (0..games)
        .into_par_iter()
        .map(|seed| {
            let mut stats = Stats::default();
            stats.add(&play_one(seed, defs));
            stats
        })
        .reduce(Stats::default, Stats::merge)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn defs() -> Arc<Defs> {
        Arc::new(Defs::load("../data/defs.ron").expect("run tests from the rogue-sim folder"))
    }

    #[test]
    fn every_method_gives_identical_statistics() {
        let defs = defs();
        let n = 40;
        let baseline = sequential(n, &defs);
        assert_eq!(baseline.games, n);
        for threads in [1, 2, 3, 7] {
            assert_eq!(
                scoped_mutex(n, &defs, threads),
                baseline,
                "mutex x{threads}"
            );
            assert_eq!(
                scoped_map_reduce(n, &defs, threads),
                baseline,
                "map-reduce x{threads}"
            );
        }
        assert_eq!(parallel(n, &defs), baseline, "rayon");
    }

    #[test]
    fn same_seed_same_result() {
        let defs = defs();
        assert_eq!(play_one(3, &defs), play_one(3, &defs));
    }

    #[test]
    fn the_bot_gets_somewhere() {
        let defs = defs();
        let stats = sequential(30, &defs);
        assert!(stats.total_depth > 0, "bot never left the first floor");
    }
}

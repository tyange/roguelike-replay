mod bot;
mod run;
mod sim;
mod stats;

use std::error::Error;
use std::process::ExitCode;
use std::sync::Arc;
use std::time::Instant;

use rogue_core::Defs;

use crate::stats::Stats;

/// 한 방식을 돌려 걸린 시간과, 통계가 순차 결과와 같은지를 한 줄로 보여 준다
fn report(name: &str, baseline: &Stats, run: impl FnOnce() -> Stats) {
    let started = Instant::now();
    let stats = run();
    let elapsed = started.elapsed().as_secs_f64();
    let verdict = if &stats == baseline {
        "statistics same"
    } else {
        "DIFFERENT"
    };
    println!("{name:<24}{elapsed:>8.2}s   {verdict}");
}

fn main() -> ExitCode {
    let games: u64 = std::env::args()
        .nth(1)
        .and_then(|arg| arg.parse().ok())
        .unwrap_or(1000);

    // rogue-sim 폴더에서 실행하므로 저장소 루트의 data 폴더는 한 단계 위에 있다
    let defs = match Defs::load("../data/defs.ron") {
        Ok(defs) => Arc::new(defs),
        Err(err) => {
            eprintln!("error: {err}");
            let mut source = err.source();
            while let Some(cause) = source {
                eprintln!("caused by: {cause}");
                source = cause.source();
            }
            return ExitCode::FAILURE;
        }
    };
    let threads =
        std::thread::available_parallelism().map_or(4, |n| u64::try_from(n.get()).unwrap_or(4));

    let started = Instant::now();
    let baseline = sim::sequential(games, &defs);
    let elapsed = started.elapsed().as_secs_f64();
    println!("{baseline}");
    println!("{:<24}{elapsed:>8.2}s", "sequential");

    report("scoped + Arc<Mutex>", &baseline, || {
        sim::scoped_mutex(games, &defs, threads)
    });
    report("scoped + map-reduce", &baseline, || {
        sim::scoped_map_reduce(games, &defs, threads)
    });
    report("rayon", &baseline, || sim::parallel(games, &defs));
    println!("({threads} threads)");

    ExitCode::SUCCESS
}

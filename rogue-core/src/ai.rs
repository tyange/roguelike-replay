//! 구현 B 에서는 판단을 `brain.rs` 의 뇌들이 맡는다. 여기에는 두 뇌가 함께 쓰는 이동 도우미만 남긴다

use crate::geom::{Dir, Point};
use crate::map::Map;

/// 목표 쪽으로 한 칸 (거리가 먼 축 우선)
pub fn step_toward(from: Point, to: Point) -> Dir {
    let (dx, dy) = (to.x - from.x, to.y - from.y);
    if dx.abs() >= dy.abs() {
        if dx > 0 { Dir::Right } else { Dir::Left }
    } else if dy > 0 {
        Dir::Down
    } else {
        Dir::Up
    }
}

/// 위협에서 가장 멀어지는, 걸어갈 수 있는 방향. 갈 곳이 없으면 None
pub fn step_away(map: &Map, from: Point, threat: Point) -> Option<Dir> {
    Dir::ALL
        .into_iter()
        .filter(|&d| map.is_walkable(from.step(d)))
        .max_by_key(|&d| {
            let p = from.step(d);
            (p.x - threat.x).pow(2) + (p.y - threat.y).pow(2)
        })
}

use std::collections::HashSet;

use crate::geom::{Point, line};
use crate::map::Map;

/// `origin`에서 `range` 안에 있고 벽에 가려지지 않은 좌표들.
/// 반경 둘레의 각 칸으로 직선을 긋고, 벽을 만나면 멈춘다
pub fn compute(map: &Map, origin: Point, range: i32) -> HashSet<Point> {
    let mut seen = HashSet::new();
    seen.insert(origin);
    for target in perimeter(origin, range) {
        for p in line(origin, target).into_iter().skip(1) {
            let (dx, dy) = (p.x - origin.x, p.y - origin.y);
            if dx * dx + dy * dy > range * range {
                break;
            }
            seen.insert(p); // 벽 자체는 보인다
            if map.blocks_sight(p) {
                break;
            }
        }
    }
    seen
}

fn perimeter(c: Point, r: i32) -> impl Iterator<Item = Point> {
    (-r..=r).flat_map(move |d| {
        [
            Point::new(c.x + d, c.y - r),
            Point::new(c.x + d, c.y + r),
            Point::new(c.x - r, c.y + d),
            Point::new(c.x + r, c.y + d),
        ]
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::Tile;

    #[test]
    fn open_room_sees_within_range_only() {
        let map = Map::room(30, 20);
        let seen = compute(&map, Point::new(15, 10), 4);
        assert!(seen.contains(&Point::new(15, 10)));
        assert!(seen.contains(&Point::new(18, 10)));
        assert!(!seen.contains(&Point::new(22, 10)));
    }

    #[test]
    fn wall_hides_what_is_behind_it() {
        let mut map = Map::room(30, 20);
        map.set(Point::new(12, 10), Tile::Wall);
        let seen = compute(&map, Point::new(10, 10), 6);
        assert!(seen.contains(&Point::new(12, 10))); // 벽 자체는 보이고
        assert!(!seen.contains(&Point::new(14, 10))); // 그 뒤는 안 보인다
    }
}

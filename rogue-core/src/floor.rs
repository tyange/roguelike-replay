use std::collections::HashSet;

use crate::defs::Defs;
use crate::dungeon;
use crate::geom::Point;
use crate::map::Map;
use crate::rng::Rng;

/// 층 하나의 "설계도". 월드를 건드리지 않는 순수한 값이라서 다른 스레드에서 만들어 보낼 수 있다.
/// 몬스터와 아이템은 정의 목록의 번호로 적어 둔다
pub struct Floor {
    pub depth: u32,
    pub map: Map,
    pub start: Point,
    pub monsters: Vec<(usize, Point)>, // (정의 목록의 번호, 위치)
    pub items: Vec<(usize, Point)>,
}

/// (시드, 깊이, 정의 목록)이 같으면 항상 같은 층이다
pub fn generate_floor(seed: u64, depth: u32, defs: &Defs) -> Floor {
    let mut rng = Rng::new(seed);
    let dungeon = dungeon::generate(40, 25, &mut rng);
    let start = dungeon
        .rooms
        .first()
        .map_or(Point::new(1, 1), |room| room.center());
    let mut used: HashSet<Point> = HashSet::from([start]);
    let mut monsters = Vec::new();
    let mut items = Vec::new();
    // 깊어질수록 몬스터가 조금씩 늘어난다
    let max_monsters = 2 + i32::try_from(depth / 2).unwrap_or(0);
    for room in dungeon.rooms.iter().skip(1) {
        for _ in 0..rng.range(0, max_monsters + 1) {
            let p = room.random_point(&mut rng);
            if defs.monsters.is_empty() || !used.insert(p) {
                continue;
            }
            let index =
                usize::try_from(rng.range(0, i32::try_from(defs.monsters.len()).unwrap_or(1)))
                    .unwrap_or(0);
            monsters.push((index, p));
        }
        for (index, def) in defs.items.iter().enumerate() {
            if rng.chance(def.chance) {
                items.push((index, room.random_point(&mut rng)));
            }
        }
    }
    Floor {
        depth,
        map: dungeon.map,
        start,
        monsters,
        items,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::defs::builtin;
    use crate::map::Tile;
    use crate::path::astar;

    fn assert_send<T: Send>() {}

    #[test]
    fn floor_and_defs_can_cross_threads() {
        assert_send::<Floor>();
        assert_send::<std::sync::Arc<Defs>>();
    }

    #[test]
    fn same_inputs_same_floor() {
        let defs = builtin();
        let a = generate_floor(8, 2, &defs);
        let b = generate_floor(8, 2, &defs);
        assert_eq!(a.map, b.map);
        assert_eq!(a.monsters, b.monsters);
        assert_eq!(a.items, b.items);
    }

    #[test]
    fn stairs_exist_and_are_reachable() {
        let defs = builtin();
        for seed in 0..100 {
            let f = generate_floor(seed, 0, &defs);
            let stairs = f.map.find(Tile::Stairs).expect("stairs");
            assert!(astar(&f.map, f.start, stairs).is_some(), "seed {seed}");
            assert_ne!(stairs, f.start);
        }
    }

    #[test]
    fn nothing_spawns_on_the_start_tile() {
        let defs = builtin();
        for seed in 0..100 {
            let f = generate_floor(seed, 3, &defs);
            assert!(f.monsters.iter().all(|(_, p)| *p != f.start));
        }
    }
}

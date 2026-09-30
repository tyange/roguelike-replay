use crate::geom::Point;
use crate::map::{Map, Tile};
use crate::rng::Rng;

/// 이보다 작은 구역은 더 쪼개지 않는다
const MIN_AREA: i32 = 9;
const MAX_DEPTH: u32 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Rect {
    pub fn center(&self) -> Point {
        Point::new(self.x + self.w / 2, self.y + self.h / 2)
    }

    /// 사각형 안의 무작위 좌표
    pub fn random_point(&self, rng: &mut Rng) -> Point {
        let x = rng.range(self.x, self.x + self.w);
        let y = rng.range(self.y, self.y + self.h);
        Point::new(x, y)
    }
}

/// 공간을 쪼갠 트리. 자식이 자기 자신 타입이라서 `Box`가 필요하다
enum Node {
    Leaf(Rect),
    Split(Box<Node>, Box<Node>),
}

pub struct Dungeon {
    pub map: Map,
    /// 만들어진 순서대로의 방 목록. 첫 방이 시작 방이다
    pub rooms: Vec<Rect>,
}

pub fn generate(width: i32, height: i32, rng: &mut Rng) -> Dungeon {
    let whole = Rect {
        x: 0,
        y: 0,
        w: width,
        h: height,
    };
    let tree = split(whole, rng, MAX_DEPTH);
    let mut map = Map::solid(width, height);
    let mut rooms = Vec::new();
    carve(&tree, &mut map, rng, &mut rooms);
    Dungeon { map, rooms }
}

fn split(area: Rect, rng: &mut Rng, depth: u32) -> Node {
    let can_cut_rows = area.h >= 2 * MIN_AREA; // 위아래로 자를 수 있나
    let can_cut_cols = area.w >= 2 * MIN_AREA; // 좌우로 자를 수 있나
    if depth == 0 || !(can_cut_rows || can_cut_cols) {
        return Node::Leaf(area);
    }
    let cut_rows = if can_cut_rows && can_cut_cols {
        rng.chance(50)
    } else {
        can_cut_rows
    };
    let (a, b) = if cut_rows {
        let at = rng.range(MIN_AREA, area.h - MIN_AREA + 1);
        (
            Rect { h: at, ..area },
            Rect {
                y: area.y + at,
                h: area.h - at,
                ..area
            },
        )
    } else {
        let at = rng.range(MIN_AREA, area.w - MIN_AREA + 1);
        (
            Rect { w: at, ..area },
            Rect {
                x: area.x + at,
                w: area.w - at,
                ..area
            },
        )
    };
    // 같은 rng 를 가변 참조로 이어서 쓴다. 두 번째 호출은 첫 번째가 쓴 다음 상태에서 시작한다
    let first = split(a, rng, depth - 1);
    let second = split(b, rng, depth - 1);
    Node::Split(Box::new(first), Box::new(second))
}

/// 트리를 훑으며 방을 파고 복도로 잇는다. 이 가지를 대표하는 좌표를 돌려준다
fn carve(node: &Node, map: &mut Map, rng: &mut Rng, rooms: &mut Vec<Rect>) -> Point {
    match node {
        Node::Leaf(area) => {
            let w = rng.range(4, area.w - 1);
            let h = rng.range(4, area.h - 1);
            // 구역 가장자리에 벽 한 칸은 남긴다
            let room = Rect {
                x: area.x + rng.range(1, area.w - w),
                y: area.y + rng.range(1, area.h - h),
                w,
                h,
            };
            for y in room.y..room.y + room.h {
                for x in room.x..room.x + room.w {
                    map.set(Point::new(x, y), Tile::Floor);
                }
            }
            rooms.push(room);
            room.center()
        }
        Node::Split(first, second) => {
            let a = carve(first, map, rng, rooms);
            let b = carve(second, map, rng, rooms);
            dig_corridor(map, a, b, rng);
            a
        }
    }
}

/// ㄱ자 복도. 가로부터 팔지 세로부터 팔지는 무작위
fn dig_corridor(map: &mut Map, a: Point, b: Point, rng: &mut Rng) {
    let (corner_x, corner_y) = if rng.chance(50) {
        (b.x, a.y)
    } else {
        (a.x, b.y)
    };
    let corner = Point::new(corner_x, corner_y);
    dig_line(map, a, corner);
    dig_line(map, corner, b);
}

/// 가로 또는 세로로 곧은 선을 판다
fn dig_line(map: &mut Map, from: Point, to: Point) {
    for x in from.x.min(to.x)..=from.x.max(to.x) {
        map.set(Point::new(x, from.y), Tile::Floor);
    }
    for y in from.y.min(to.y)..=from.y.max(to.y) {
        map.set(Point::new(to.x, y), Tile::Floor);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::path::astar;

    #[test]
    fn same_seed_same_dungeon() {
        let a = generate(40, 25, &mut Rng::new(5));
        let b = generate(40, 25, &mut Rng::new(5));
        assert_eq!(a.rooms, b.rooms);
        assert_eq!(a.map, b.map);
    }

    #[test]
    fn different_seeds_make_different_dungeons() {
        let a = generate(40, 25, &mut Rng::new(1));
        let b = generate(40, 25, &mut Rng::new(2));
        assert_ne!(a.map, b.map);
    }

    #[test]
    fn every_room_is_reachable_from_the_first() {
        for seed in 0..200 {
            let d = generate(40, 25, &mut Rng::new(seed));
            assert!(d.rooms.len() >= 2, "seed {seed}: too few rooms");
            let start = d.rooms[0].center();
            for room in &d.rooms {
                assert!(
                    astar(&d.map, start, room.center()).is_some(),
                    "seed {seed}: room at {:?} is cut off",
                    room.center()
                );
            }
        }
    }

    #[test]
    fn rooms_stay_inside_the_border() {
        for seed in 0..200 {
            let d = generate(40, 25, &mut Rng::new(seed));
            for r in &d.rooms {
                assert!(r.x >= 1 && r.y >= 1 && r.x + r.w <= 39 && r.y + r.h <= 24);
            }
        }
    }
}

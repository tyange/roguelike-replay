use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

use crate::geom::{Dir, Point};
use crate::map::Map;

/// 열린 목록에 넣을 노드. BinaryHeap은 큰 것부터 꺼내므로 비용이 작을수록 "크게" 비교한다
#[derive(Debug, PartialEq, Eq)]
struct Node {
    f: u32,
    pos: Point,
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .f
            .cmp(&self.f)
            .then_with(|| (self.pos.x, self.pos.y).cmp(&(other.pos.x, other.pos.y)))
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// 맨해튼 거리
fn heuristic(a: Point, b: Point) -> u32 {
    a.x.abs_diff(b.x) + a.y.abs_diff(b.y)
}

/// `start`에서 `goal`까지의 경로. 출발점은 빼고 도착점은 포함한다. 갈 수 없으면 None
pub fn astar(map: &Map, start: Point, goal: Point) -> Option<Vec<Point>> {
    let mut open = BinaryHeap::new();
    let mut came_from: HashMap<Point, Point> = HashMap::new();
    let mut cost: HashMap<Point, u32> = HashMap::new();
    cost.insert(start, 0);
    open.push(Node {
        f: heuristic(start, goal),
        pos: start,
    });

    while let Some(Node { pos, .. }) = open.pop() {
        if pos == goal {
            return Some(reconstruct(&came_from, start, goal));
        }
        let g = cost.get(&pos).copied()?;
        for dir in Dir::ALL {
            let next = pos.step(dir);
            if !map.is_walkable(next) {
                continue;
            }
            let new_g = g + 1;
            if cost.get(&next).is_none_or(|&old| new_g < old) {
                cost.insert(next, new_g);
                came_from.insert(next, pos);
                open.push(Node {
                    f: new_g + heuristic(next, goal),
                    pos: next,
                });
            }
        }
    }
    None
}

fn reconstruct(came_from: &HashMap<Point, Point>, start: Point, goal: Point) -> Vec<Point> {
    let mut path = Vec::new();
    let mut cur = goal;
    while cur != start {
        path.push(cur);
        match came_from.get(&cur) {
            Some(&prev) => cur = prev,
            None => break,
        }
    }
    path.reverse();
    path
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::Tile;

    #[test]
    fn straight_path_in_open_room() {
        let map = Map::room(30, 20);
        let path = astar(&map, Point::new(1, 1), Point::new(5, 1)).expect("길이 있다");
        assert_eq!(path.len(), 4);
        assert_eq!(path.last(), Some(&Point::new(5, 1)));
        assert_eq!(path.first(), Some(&Point::new(2, 1)));
    }

    #[test]
    fn same_start_and_goal_is_empty_path() {
        let map = Map::room(30, 20);
        assert_eq!(
            astar(&map, Point::new(3, 3), Point::new(3, 3)),
            Some(vec![])
        );
    }

    #[test]
    fn goes_around_a_wall_with_a_gap() {
        let mut map = Map::room(30, 20);
        for y in 1..19 {
            map.set(Point::new(10, y), Tile::Wall);
        }
        map.set(Point::new(10, 15), Tile::Floor); // 틈
        let path = astar(&map, Point::new(5, 5), Point::new(15, 5)).expect("틈으로 돌아간다");
        assert!(path.contains(&Point::new(10, 15)));
        assert!(path.iter().all(|&p| map.is_walkable(p)));
    }

    #[test]
    fn blocked_map_has_no_path() {
        let mut map = Map::room(30, 20);
        for y in 1..19 {
            map.set(Point::new(10, y), Tile::Wall);
        }
        assert_eq!(astar(&map, Point::new(5, 5), Point::new(15, 5)), None);
    }
}

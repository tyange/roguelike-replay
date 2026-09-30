use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub const fn new(x: i32, y: i32) -> Point {
        Point { x, y }
    }

    /// 이 좌표에서 `dir` 방향으로 한 칸 옮긴 좌표
    pub fn step(self, dir: Dir) -> Point {
        let (dx, dy) = dir.delta();
        Point::new(self.x + dx, self.y + dy)
    }

    /// 두 좌표 사이 거리의 제곱. 제곱근 없이 거리를 비교할 때 쓴다
    pub fn dist2(self, other: Point) -> i32 {
        (self.x - other.x).pow(2) + (self.y - other.y).pow(2)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Dir {
    Up,
    Down,
    Left,
    Right,
}

impl Dir {
    pub const ALL: [Dir; 4] = [Dir::Up, Dir::Down, Dir::Left, Dir::Right];

    pub fn delta(self) -> (i32, i32) {
        match self {
            Dir::Up => (0, -1),
            Dir::Down => (0, 1),
            Dir::Left => (-1, 0),
            Dir::Right => (1, 0),
        }
    }
}

impl Dir {
    /// 이웃한 두 좌표 사이의 방향. 이웃이 아니면 None
    pub fn between(from: Point, to: Point) -> Option<Dir> {
        Dir::ALL.into_iter().find(|&d| from.step(d) == to)
    }
}

/// 두 점을 잇는 직선 위의 칸들(양 끝 포함). 브레젠험 알고리즘
pub fn line(a: Point, b: Point) -> Vec<Point> {
    let dx = (b.x - a.x).abs();
    let dy = -(b.y - a.y).abs();
    let sx = if a.x < b.x { 1 } else { -1 };
    let sy = if a.y < b.y { 1 } else { -1 };
    let mut err = dx + dy;
    let mut p = a;
    let mut out = vec![p];
    while p != b {
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            p.x += sx;
        }
        if e2 <= dx {
            err += dx;
            p.y += sy;
        }
        out.push(p);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_includes_both_ends() {
        let l = line(Point::new(0, 0), Point::new(4, 2));
        assert_eq!(l.first(), Some(&Point::new(0, 0)));
        assert_eq!(l.last(), Some(&Point::new(4, 2)));
        assert_eq!(l.len(), 5);
    }

    #[test]
    fn line_of_one_point() {
        assert_eq!(
            line(Point::new(3, 3), Point::new(3, 3)),
            vec![Point::new(3, 3)]
        );
    }

    #[test]
    fn between_neighbors() {
        assert_eq!(
            Dir::between(Point::new(1, 1), Point::new(2, 1)),
            Some(Dir::Right)
        );
        assert_eq!(Dir::between(Point::new(1, 1), Point::new(3, 1)), None);
    }
}

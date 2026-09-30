use serde::{Deserialize, Serialize};

use crate::geom::Point;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tile {
    Wall,
    Floor,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Map {
    width: i32,
    height: i32,
    tiles: Vec<Tile>,
    /// 플레이어가 한 번이라도 본 칸
    explored: Vec<bool>,
}

impl Map {
    /// 테두리는 벽, 안쪽은 바닥인 방
    pub fn room(width: i32, height: i32) -> Map {
        let len = usize::try_from(width * height).unwrap_or(0);
        let mut map = Map {
            width,
            height,
            tiles: vec![Tile::Wall; len],
            explored: vec![false; len],
        };
        for y in 1..height - 1 {
            for x in 1..width - 1 {
                map.set(Point::new(x, y), Tile::Floor);
            }
        }
        map
    }

    /// 전부 벽인 맵. 던전 생성기가 여기에 방과 복도를 판다
    pub fn solid(width: i32, height: i32) -> Map {
        let len = usize::try_from(width * height).unwrap_or(0);
        Map {
            width,
            height,
            tiles: vec![Tile::Wall; len],
            explored: vec![false; len],
        }
    }

    pub fn width(&self) -> i32 {
        self.width
    }

    pub fn height(&self) -> i32 {
        self.height
    }

    /// 좌표 → 1차원 인덱스. 맵 밖이면 None
    fn index(&self, p: Point) -> Option<usize> {
        if p.x < 0 || p.y < 0 || p.x >= self.width || p.y >= self.height {
            return None;
        }
        usize::try_from(p.y * self.width + p.x).ok()
    }

    pub fn tile(&self, p: Point) -> Option<Tile> {
        self.index(p).and_then(|i| self.tiles.get(i)).copied()
    }

    pub fn is_walkable(&self, p: Point) -> bool {
        matches!(self.tile(p), Some(Tile::Floor))
    }

    /// 시야를 가리는가. 벽이거나 맵 밖이면 가린다
    pub fn blocks_sight(&self, p: Point) -> bool {
        matches!(self.tile(p), Some(Tile::Wall) | None)
    }

    pub fn is_explored(&self, p: Point) -> bool {
        self.index(p)
            .and_then(|i| self.explored.get(i))
            .copied()
            .unwrap_or(false)
    }

    pub fn mark_explored(&mut self, p: Point) {
        if let Some(e) = self.index(p).and_then(|i| self.explored.get_mut(i)) {
            *e = true;
        }
    }

    /// 타일을 바꾼다. 맵 밖이면 false
    pub fn set(&mut self, p: Point, tile: Tile) -> bool {
        match self.index(p).and_then(|i| self.tiles.get_mut(i)) {
            Some(t) => {
                *t = tile;
                true
            }
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_in_range() {
        let map = Map::room(10, 8);
        assert_eq!(map.index(Point::new(0, 0)), Some(0));
        assert_eq!(map.index(Point::new(3, 2)), Some(23));
        assert_eq!(map.index(Point::new(9, 7)), Some(79));
    }

    #[test]
    fn index_out_of_range() {
        let map = Map::room(10, 8);
        assert_eq!(map.index(Point::new(-1, 0)), None);
        assert_eq!(map.index(Point::new(0, -1)), None);
        assert_eq!(map.index(Point::new(10, 0)), None);
        assert_eq!(map.index(Point::new(0, 8)), None);
    }

    #[test]
    fn room_has_walls_outside_and_floor_inside() {
        let map = Map::room(10, 8);
        assert_eq!(map.tile(Point::new(0, 3)), Some(Tile::Wall));
        assert_eq!(map.tile(Point::new(9, 3)), Some(Tile::Wall));
        assert_eq!(map.tile(Point::new(5, 4)), Some(Tile::Floor));
        assert_eq!(map.tile(Point::new(-5, 4)), None);
    }

    #[test]
    fn walkable_is_false_outside_map() {
        let map = Map::room(10, 8);
        assert!(map.is_walkable(Point::new(5, 4)));
        assert!(!map.is_walkable(Point::new(0, 0)));
        assert!(!map.is_walkable(Point::new(-1, -1)));
    }
}

use crate::action::Action;
use crate::geom::Point;
use crate::map::Map;

pub struct Game {
    map: Map,
    player: Point,
}

impl Game {
    pub fn new() -> Game {
        Game {
            map: Map::room(30, 20),
            player: Point::new(15, 10),
        }
    }

    pub fn map(&self) -> &Map {
        &self.map
    }

    pub fn player(&self) -> Point {
        self.player
    }

    /// 행동 하나를 받아 게임을 한 턴 진행한다. core의 유일한 입구
    pub fn step(&mut self, action: Action) {
        match action {
            Action::Wait => {}
            Action::Move(dir) => {
                let to = self.player.step(dir);
                if self.map.is_walkable(to) {
                    self.player = to;
                }
            }
        }
    }
}

impl Default for Game {
    fn default() -> Self {
        Game::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geom::Dir;

    #[test]
    fn move_into_floor() {
        let mut game = Game::new();
        game.step(Action::Move(Dir::Right));
        assert_eq!(game.player(), Point::new(16, 10));
    }

    #[test]
    fn move_into_wall_does_nothing() {
        let mut game = Game::new();
        for _ in 0..30 {
            game.step(Action::Move(Dir::Left));
        }
        assert_eq!(game.player(), Point::new(1, 10));
    }

    #[test]
    fn wait_changes_nothing() {
        let mut game = Game::new();
        game.step(Action::Wait);
        assert_eq!(game.player(), Point::new(15, 10));
    }
}

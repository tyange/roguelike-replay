use crate::action::Action;
use crate::arena::{Arena, Creature, EntityId};
use crate::geom::{Dir, Point};
use crate::map::Map;

pub struct Game {
    map: Map,
    creatures: Arena,
    player: EntityId,
}

impl Game {
    pub fn new() -> Game {
        let mut creatures = Arena::new();
        let player = creatures.spawn(Creature {
            pos: Point::new(15, 10),
            hp: 20,
            glyph: '@',
            target: None,
        });
        for pos in [Point::new(5, 5), Point::new(24, 4), Point::new(20, 15)] {
            creatures.spawn(Creature {
                pos,
                hp: 6,
                glyph: 'g',
                target: Some(player),
            });
        }
        Game {
            map: Map::room(30, 20),
            creatures,
            player,
        }
    }

    pub fn map(&self) -> &Map {
        &self.map
    }

    pub fn is_over(&self) -> bool {
        self.creatures.get(self.player).is_none()
    }

    pub fn player_hp(&self) -> i32 {
        self.creatures.get(self.player).map_or(0, |c| c.hp)
    }

    /// 화면에 그릴 (위치, 글자) 목록
    pub fn glyphs(&self) -> Vec<(Point, char)> {
        self.creatures
            .ids()
            .into_iter()
            .filter_map(|id| self.creatures.get(id))
            .map(|c| (c.pos, c.glyph))
            .collect()
    }

    /// 행동 하나를 받아 게임을 한 턴 진행한다. core의 유일한 입구
    pub fn step(&mut self, action: Action) {
        if self.is_over() {
            return;
        }
        self.perform(self.player, action);
        self.monsters_act();
    }

    fn perform(&mut self, actor: EntityId, action: Action) {
        let Action::Move(dir) = action else {
            return; // Wait
        };
        let Some(from) = self.creatures.get(actor).map(|c| c.pos) else {
            return;
        };
        let to = from.step(dir);
        match self.creature_at(to) {
            Some(other) => self.attack(actor, other),
            None if self.map.is_walkable(to) => {
                if let Some(c) = self.creatures.get_mut(actor) {
                    c.pos = to;
                }
            }
            None => {}
        }
    }

    fn creature_at(&self, p: Point) -> Option<EntityId> {
        self.creatures
            .ids()
            .into_iter()
            .find(|&id| self.creatures.get(id).is_some_and(|c| c.pos == p))
    }

    fn attack(&mut self, attacker: EntityId, defender: EntityId) {
        // 플레이어와 몬스터 사이에서만 싸운다 (몬스터끼리는 서로 막기만 한다)
        if attacker != self.player && defender != self.player {
            return;
        }
        let dead = match self.creatures.get_mut(defender) {
            Some(c) => {
                c.hp -= 1;
                c.hp <= 0
            }
            None => false,
        };
        if dead {
            self.creatures.despawn(defender);
        }
    }

    fn monsters_act(&mut self) {
        for id in self.creatures.ids() {
            if id == self.player {
                continue;
            }
            let Some(target) = self.creatures.get(id).and_then(|c| c.target) else {
                continue;
            };
            // 대상이 이미 사라졌으면 None이 나온다
            let Some(goal) = self.creatures.get(target).map(|c| c.pos) else {
                continue;
            };
            let Some(from) = self.creatures.get(id).map(|c| c.pos) else {
                continue;
            };
            self.perform(id, Action::Move(step_toward(from, goal)));
        }
    }
}

impl Default for Game {
    fn default() -> Self {
        Game::new()
    }
}

/// 목표 쪽으로 한 칸 (거리가 먼 축 우선)
fn step_toward(from: Point, to: Point) -> Dir {
    let (dx, dy) = (to.x - from.x, to.y - from.y);
    if dx.abs() >= dy.abs() {
        if dx > 0 { Dir::Right } else { Dir::Left }
    } else if dy > 0 {
        Dir::Down
    } else {
        Dir::Up
    }
}

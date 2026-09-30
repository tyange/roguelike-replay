use crate::action::Action;
use crate::entity::EntityId;
use crate::geom::{Dir, Point};
use crate::map::Map;
use crate::world::World;

pub struct Game {
    map: Map,
    world: World,
    player: EntityId,
}

impl Game {
    pub fn new() -> Game {
        let mut world = World::default();
        let player = world.spawn_player(Point::new(15, 10));
        for pos in [Point::new(5, 5), Point::new(24, 4), Point::new(20, 15)] {
            world.spawn_goblin(pos, player);
        }
        Game {
            map: Map::room(30, 20),
            world,
            player,
        }
    }

    pub fn map(&self) -> &Map {
        &self.map
    }

    pub fn world(&self) -> &World {
        &self.world
    }

    pub fn player(&self) -> EntityId {
        self.player
    }

    pub fn is_over(&self) -> bool {
        !self.world.entities.is_alive(self.player)
    }

    pub fn player_hp(&self) -> i32 {
        self.world.healths.get(self.player).map_or(0, |h| h.hp)
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
        let Some(from) = self.world.positions.get(actor).map(|p| p.0) else {
            return;
        };
        let to = from.step(dir);
        match self.world.blocker_at(to) {
            Some(other) => self.attack(actor, other),
            None if self.map.is_walkable(to) => {
                if let Some(p) = self.world.positions.get_mut(actor) {
                    p.0 = to;
                }
            }
            None => {}
        }
    }

    fn attack(&mut self, attacker: EntityId, defender: EntityId) {
        // 플레이어와 몬스터 사이에서만 싸운다 (몬스터끼리는 서로 막기만 한다)
        if attacker != self.player && defender != self.player {
            return;
        }
        let dead = match self.world.healths.get_mut(defender) {
            Some(h) => {
                h.hp -= 1;
                h.hp <= 0
            }
            None => false,
        };
        if dead {
            self.world.despawn(defender);
        }
    }

    fn monsters_act(&mut self) {
        // 순회하면서 perform 으로 월드를 고쳐야 하므로, ID 를 먼저 모아 빌림을 끝낸다
        let ids: Vec<EntityId> = self.world.monsters.iter().map(|(id, _)| id).collect();
        for id in ids {
            let Some(target) = self.world.targets.get(id).map(|t| t.0) else {
                continue;
            };
            // 대상이 이미 사라졌으면 None이 나온다
            let Some(goal) = self.world.positions.get(target).map(|p| p.0) else {
                continue;
            };
            let Some(from) = self.world.positions.get(id).map(|p| p.0) else {
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

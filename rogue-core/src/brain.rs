use crate::action::Action;
use crate::ai::{step_away, step_toward};
use crate::comps::Ai;
use crate::entity::EntityId;
use crate::geom::Point;
use crate::map::Map;
use crate::world::World;

/// 구현 B: 몬스터의 판단을 트레이트 객체로. 새 종류는 구조체 하나와 impl 하나로 끝난다
pub trait Brain {
    fn decide(&mut self, world: &World, map: &Map, me: EntityId, player: EntityId) -> Action;
}

pub struct ChaserBrain;

pub struct CowardBrain {
    pub flee_below: i32,
}

/// 데이터(Ai)에서 뇌를 만든다
pub fn brain_for(ai: Ai) -> Box<dyn Brain> {
    match ai {
        Ai::Chaser => Box::new(ChaserBrain),
        Ai::Coward { flee_below } => Box::new(CowardBrain { flee_below }),
    }
}

/// 나와 플레이어의 위치. 하나라도 없으면 None
fn positions(world: &World, me: EntityId, player: EntityId) -> Option<(Point, Point)> {
    let from = world.positions.get(me)?.0;
    let goal = world.positions.get(player)?.0;
    Some((from, goal))
}

impl Brain for ChaserBrain {
    fn decide(&mut self, world: &World, _map: &Map, me: EntityId, player: EntityId) -> Action {
        match positions(world, me, player) {
            Some((from, goal)) => Action::Move(step_toward(from, goal)),
            None => Action::Wait,
        }
    }
}

impl Brain for CowardBrain {
    fn decide(&mut self, world: &World, map: &Map, me: EntityId, player: EntityId) -> Action {
        let Some((from, goal)) = positions(world, me, player) else {
            return Action::Wait;
        };
        let hp = world.healths.get(me).map_or(0, |h| h.hp);
        if hp < self.flee_below {
            step_away(map, from, goal).map_or(Action::Wait, Action::Move)
        } else {
            Action::Move(step_toward(from, goal))
        }
    }
}

/// 모든 몬스터의 뇌에게 행동을 묻는다
pub fn run_brains(world: &mut World, map: &Map, player: EntityId) -> Vec<(EntityId, Action)> {
    let ids: Vec<EntityId> = world.monsters.iter().map(|(id, _)| id).collect();
    let mut out = Vec::new();
    for id in ids {
        // 잠깐 꺼냈다가 다시 넣는다. 꺼내 있는 동안에는 월드를 자유롭게 읽을 수 있다
        let Some(mut brain) = world.brains.remove(id) else {
            continue;
        };
        let action = brain.decide(world, map, id, player);
        world.brains.insert(id, brain);
        out.push((id, action));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geom::Dir;
    use crate::map::Tile;

    /// 한 마리의 뇌에게만 묻는다
    fn ask(world: &mut World, map: &Map, me: EntityId, player: EntityId) -> Option<Action> {
        run_brains(world, map, player)
            .into_iter()
            .find(|&(id, _)| id == me)
            .map(|(_, action)| action)
    }

    fn setup(monster_at: Point) -> (World, Map, EntityId, EntityId) {
        let mut world = World::default();
        let player = world.spawn_player(Point::new(10, 10));
        let kobold = world.spawn_kobold(monster_at);
        (world, Map::room(30, 20), player, kobold)
    }

    #[test]
    fn chaser_approaches() {
        let mut world = World::default();
        let player = world.spawn_player(Point::new(10, 10));
        let goblin = world.spawn_goblin(Point::new(4, 10));
        let map = Map::room(30, 20);
        assert_eq!(
            ask(&mut world, &map, goblin, player),
            Some(Action::Move(Dir::Right))
        );
    }

    #[test]
    fn healthy_coward_approaches() {
        let (mut world, map, player, kobold) = setup(Point::new(4, 10));
        assert_eq!(
            ask(&mut world, &map, kobold, player),
            Some(Action::Move(Dir::Right))
        );
    }

    #[test]
    fn hurt_coward_flees() {
        let (mut world, map, player, kobold) = setup(Point::new(4, 10));
        if let Some(h) = world.healths.get_mut(kobold) {
            h.hp = 2;
        }
        assert_eq!(
            ask(&mut world, &map, kobold, player),
            Some(Action::Move(Dir::Left))
        );
    }

    #[test]
    fn cornered_coward_waits() {
        let mut world = World::default();
        let player = world.spawn_player(Point::new(2, 2));
        let kobold = world.spawn_kobold(Point::new(1, 1));
        if let Some(h) = world.healths.get_mut(kobold) {
            h.hp = 1;
        }
        let mut map = Map::room(30, 20);
        map.set(Point::new(2, 1), Tile::Wall);
        map.set(Point::new(1, 2), Tile::Wall);
        assert_eq!(ask(&mut world, &map, kobold, player), Some(Action::Wait));
    }

    #[test]
    fn brains_go_back_after_being_asked() {
        let (mut world, map, player, kobold) = setup(Point::new(4, 10));
        run_brains(&mut world, &map, player);
        assert!(world.brains.get(kobold).is_some());
    }
}

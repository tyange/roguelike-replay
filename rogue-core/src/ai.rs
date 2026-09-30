use crate::action::Action;
use crate::comps::Ai;
use crate::entity::EntityId;
use crate::geom::{Dir, Point};
use crate::map::Map;
use crate::world::World;

/// 월드를 읽기만 하고 행동 하나를 고른다. 실제 적용은 플레이어와 같은 경로를 탄다
pub fn decide(world: &World, map: &Map, me: EntityId, player: EntityId) -> Action {
    let Some(&ai) = world.ais.get(me) else {
        return Action::Wait;
    };
    let (Some(from), Some(goal)) = (
        world.positions.get(me).map(|p| p.0),
        world.positions.get(player).map(|p| p.0),
    ) else {
        return Action::Wait;
    };
    match ai {
        Ai::Chaser => Action::Move(step_toward(from, goal)),
        Ai::Coward { flee_below } => {
            let hp = world.healths.get(me).map_or(0, |h| h.hp);
            if hp < flee_below {
                step_away(map, from, goal).map_or(Action::Wait, Action::Move)
            } else {
                Action::Move(step_toward(from, goal))
            }
        }
    }
}

/// 목표 쪽으로 한 칸 (거리가 먼 축 우선)
pub fn step_toward(from: Point, to: Point) -> Dir {
    let (dx, dy) = (to.x - from.x, to.y - from.y);
    if dx.abs() >= dy.abs() {
        if dx > 0 { Dir::Right } else { Dir::Left }
    } else if dy > 0 {
        Dir::Down
    } else {
        Dir::Up
    }
}

/// 위협에서 가장 멀어지는, 걸어갈 수 있는 방향. 갈 곳이 없으면 None
fn step_away(map: &Map, from: Point, threat: Point) -> Option<Dir> {
    Dir::ALL
        .into_iter()
        .filter(|&d| map.is_walkable(from.step(d)))
        .max_by_key(|&d| {
            let p = from.step(d);
            (p.x - threat.x).pow(2) + (p.y - threat.y).pow(2)
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::Tile;

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
            decide(&world, &map, goblin, player),
            Action::Move(Dir::Right)
        );
    }

    #[test]
    fn healthy_coward_approaches() {
        let (world, map, player, kobold) = setup(Point::new(4, 10));
        assert_eq!(
            decide(&world, &map, kobold, player),
            Action::Move(Dir::Right)
        );
    }

    #[test]
    fn hurt_coward_flees() {
        let (mut world, map, player, kobold) = setup(Point::new(4, 10));
        if let Some(h) = world.healths.get_mut(kobold) {
            h.hp = 2;
        }
        assert_eq!(
            decide(&world, &map, kobold, player),
            Action::Move(Dir::Left)
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
        assert_eq!(decide(&world, &map, kobold, player), Action::Wait);
    }
}

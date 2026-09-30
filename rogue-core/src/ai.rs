use crate::action::Action;
use crate::comps::Ai;
use crate::entity::EntityId;
use crate::geom::{Dir, Point};
use crate::map::Map;
use crate::path::astar;
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
    // 플레이어가 내 시야에 있을 때만 반응한다
    let sees_player = world
        .viewsheds
        .get(me)
        .is_some_and(|v| v.visible.contains(&goal));
    if !sees_player {
        return Action::Wait;
    }
    match ai {
        Ai::Chaser => chase(map, from, goal),
        Ai::Coward { flee_below } => {
            let hp = world.healths.get(me).map_or(0, |h| h.hp);
            if hp < flee_below {
                step_away(map, from, goal).map_or(Action::Wait, Action::Move)
            } else {
                chase(map, from, goal)
            }
        }
    }
}

/// 길찾기로 구한 경로의 첫 걸음
fn chase(map: &Map, from: Point, goal: Point) -> Action {
    astar(map, from, goal)
        .and_then(|path| path.first().copied())
        .and_then(|next| Dir::between(from, next))
        .map_or(Action::Wait, Action::Move)
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
    use crate::fov;
    use crate::map::Tile;

    /// 모든 엔티티의 시야를 계산해 둔다
    fn look(world: &mut World, map: &Map) {
        for (id, vs) in world.viewsheds.iter_mut() {
            if let Some(pos) = world.positions.get(id) {
                vs.visible = fov::compute(map, pos.0, vs.range);
                vs.dirty = false;
            }
        }
    }

    fn setup(monster_at: Point) -> (World, Map, EntityId, EntityId) {
        let mut world = World::default();
        let player = world.spawn_player(Point::new(10, 10));
        let kobold = world.spawn_kobold(monster_at);
        let map = Map::room(30, 20);
        look(&mut world, &map);
        (world, map, player, kobold)
    }

    #[test]
    fn chaser_approaches() {
        let mut world = World::default();
        let player = world.spawn_player(Point::new(10, 10));
        let goblin = world.spawn_goblin(Point::new(6, 10));
        let map = Map::room(30, 20);
        look(&mut world, &map);
        assert_eq!(
            decide(&world, &map, goblin, player),
            Action::Move(Dir::Right)
        );
    }

    #[test]
    fn healthy_coward_approaches() {
        let (world, map, player, kobold) = setup(Point::new(6, 10));
        assert_eq!(
            decide(&world, &map, kobold, player),
            Action::Move(Dir::Right)
        );
    }

    #[test]
    fn hurt_coward_flees() {
        let (mut world, map, player, kobold) = setup(Point::new(6, 10));
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
        look(&mut world, &map);
        assert_eq!(decide(&world, &map, kobold, player), Action::Wait);
    }

    #[test]
    fn chaser_waits_when_it_cannot_see() {
        let mut world = World::default();
        let player = world.spawn_player(Point::new(10, 10));
        let goblin = world.spawn_goblin(Point::new(1, 1));
        let map = Map::room(30, 20);
        look(&mut world, &map);
        assert_eq!(decide(&world, &map, goblin, player), Action::Wait);
    }

    #[test]
    fn chaser_goes_around_a_wall() {
        let mut world = World::default();
        let player = world.spawn_player(Point::new(10, 10));
        let goblin = world.spawn_goblin(Point::new(5, 10));
        let mut map = Map::room(30, 20);
        for y in 6..=14 {
            map.set(Point::new(6, y), Tile::Wall);
        }
        look(&mut world, &map);
        // 벽이 직선 시야를 막으므로, "방금까지 보였다"를 흉내 낸다
        if let Some(vs) = world.viewsheds.get_mut(goblin) {
            vs.visible.insert(Point::new(10, 10));
        }
        assert!(matches!(
            decide(&world, &map, goblin, player),
            Action::Move(Dir::Up | Dir::Down)
        ));
    }
}

use crate::entity::EntityId;
use crate::event::Event;
use crate::world::World;

/// 공격력에서 방어력을 뺀 피해. 최소 1
pub fn damage(power: i32, defense: i32) -> i32 {
    (power - defense).max(1)
}

/// 방법 3: 이벤트. 아무것도 바꾸지 않고, 일어나야 할 일만 돌려준다
pub fn attack_event(world: &World, attacker: EntityId, defender: EntityId) -> Option<Event> {
    let power = world.combats.get(attacker)?.power;
    let defense = world.combats.get(defender)?.defense;
    Some(Event::Damage {
        target: defender,
        amount: damage(power, defense),
        source: attacker,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geom::Point;

    // 연습: 흡혈 공격(방어자는 피해를 입고, 공격자는 그 절반을 회복)을 세 가지 방법으로 구현해 비교한다.
    // 공격자와 방어자의 Health가 같은 저장소 안에 있어서 둘 다 가변으로 빌려야 하는 상황이다.

    fn setup() -> (World, EntityId, EntityId) {
        let mut world = World::default();
        let player = world.spawn_player(Point::new(1, 1));
        let goblin = world.spawn_goblin(Point::new(2, 1), player);
        if let Some(h) = world.healths.get_mut(player) {
            h.hp = 20; // 회복할 여유를 만든다
        }
        (world, player, goblin)
    }

    /// 방법 1: 값을 먼저 꺼내기
    fn lifesteal_copy_out(world: &mut World, attacker: EntityId, defender: EntityId) -> Option<()> {
        let power = world.combats.get(attacker)?.power;
        let defense = world.combats.get(defender)?.defense;
        let amount = damage(power, defense);
        world.healths.get_mut(defender)?.hp -= amount; // 이 빌림은 여기서 끝난다
        let h = world.healths.get_mut(attacker)?;
        h.hp = (h.hp + amount / 2).min(h.max);
        Some(())
    }

    /// 방법 2: 서로 다른 두 칸을 동시에 가변으로 빌리기
    fn lifesteal_disjoint(world: &mut World, attacker: EntityId, defender: EntityId) -> Option<()> {
        let power = world.combats.get(attacker)?.power;
        let defense = world.combats.get(defender)?.defense;
        let amount = damage(power, defense);
        let (a, d) = world.healths.get_two_mut(attacker, defender)?; // 두 칸을 한 번에 빌린다
        d.hp -= amount;
        a.hp = (a.hp + amount / 2).min(a.max);
        Some(())
    }

    /// 방법 3: 이벤트. 흡혈은 "피해"와 "회복" 두 이벤트의 조합일 뿐이다
    fn lifesteal_events(
        world: &World,
        attacker: EntityId,
        defender: EntityId,
    ) -> Vec<(EntityId, i32)> {
        let Some(Event::Damage {
            target,
            amount,
            source,
        }) = attack_event(world, attacker, defender)
        else {
            return Vec::new();
        };
        vec![(target, -amount), (source, amount / 2)]
    }

    #[test]
    fn all_three_methods_agree() {
        let (mut w1, p1, g1) = setup();
        lifesteal_copy_out(&mut w1, p1, g1);
        let (mut w2, p2, g2) = setup();
        lifesteal_disjoint(&mut w2, p2, g2);
        let (w3, p3, g3) = setup();
        let changes = lifesteal_events(&w3, p3, g3);

        // 플레이어(공격 5) → 고블린(방어 1): 피해 4, 흡혈 2
        assert_eq!(w1.healths.get(g1).map(|h| h.hp), Some(6));
        assert_eq!(w1.healths.get(p1).map(|h| h.hp), Some(22));
        assert_eq!(w2.healths.get(g2).map(|h| h.hp), Some(6));
        assert_eq!(w2.healths.get(p2).map(|h| h.hp), Some(22));
        assert_eq!(changes, vec![(g3, -4), (p3, 2)]);
    }

    #[test]
    fn minimum_damage_is_one() {
        assert_eq!(damage(1, 10), 1);
    }
}

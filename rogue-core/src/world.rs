use crate::brain::{Brain, brain_for};
use crate::comps::{Ai, BlocksTile, Combat, Glyph, Health, Monster, Name, Player, Position, Rgb};
use crate::entity::{Entities, EntityId};
use crate::geom::Point;
use crate::store::Store;

#[derive(Default)]
pub struct World {
    pub entities: Entities,
    pub positions: Store<Position>,
    pub glyphs: Store<Glyph>,
    pub healths: Store<Health>,
    pub names: Store<Name>,
    pub blockers: Store<BlocksTile>,
    pub players: Store<Player>,
    pub monsters: Store<Monster>,
    pub ais: Store<Ai>,
    pub combats: Store<Combat>,
    /// 구현 B: 몬스터의 뇌. 트레이트 객체라서 Clone, Debug 를 derive 할 수 없다
    pub brains: Store<Box<dyn Brain>>,
}

impl World {
    /// 엔티티와 모든 컴포넌트를 지운다
    pub fn despawn(&mut self, id: EntityId) {
        self.entities.despawn(id);
        self.positions.remove(id);
        self.glyphs.remove(id);
        self.healths.remove(id);
        self.names.remove(id);
        self.blockers.remove(id);
        self.players.remove(id);
        self.monsters.remove(id);
        self.ais.remove(id);
        self.combats.remove(id);
        self.brains.remove(id);
    }

    pub fn spawn_player(&mut self, pos: Point) -> EntityId {
        let id = self.entities.spawn();
        self.positions.insert(id, Position(pos));
        self.glyphs.insert(
            id,
            Glyph {
                ch: '@',
                color: Rgb {
                    r: 255,
                    g: 221,
                    b: 51,
                },
            },
        );
        self.healths.insert(id, Health { hp: 30, max: 30 });
        self.combats.insert(
            id,
            Combat {
                power: 5,
                defense: 2,
            },
        );
        self.names.insert(id, Name("Player".to_string()));
        self.blockers.insert(id, BlocksTile);
        self.players.insert(id, Player);
        id
    }

    pub fn spawn_goblin(&mut self, pos: Point) -> EntityId {
        self.spawn_monster(
            pos,
            'g',
            Rgb {
                r: 80,
                g: 200,
                b: 80,
            },
            "Goblin",
            (10, 4, 1),
            Ai::Chaser,
        )
    }

    pub fn spawn_kobold(&mut self, pos: Point) -> EntityId {
        self.spawn_monster(
            pos,
            'k',
            Rgb {
                r: 230,
                g: 140,
                b: 60,
            },
            "Kobold",
            (8, 3, 0),
            Ai::Coward { flee_below: 3 },
        )
    }

    /// stats = (체력, 공격력, 방어력)
    fn spawn_monster(
        &mut self,
        pos: Point,
        ch: char,
        color: Rgb,
        name: &str,
        stats: (i32, i32, i32),
        ai: Ai,
    ) -> EntityId {
        let (hp, power, defense) = stats;
        let id = self.entities.spawn();
        self.positions.insert(id, Position(pos));
        self.glyphs.insert(id, Glyph { ch, color });
        self.healths.insert(id, Health { hp, max: hp });
        self.combats.insert(id, Combat { power, defense });
        self.names.insert(id, Name(name.to_string()));
        self.blockers.insert(id, BlocksTile);
        self.monsters.insert(id, Monster);
        self.ais.insert(id, ai);
        self.brains.insert(id, brain_for(ai));
        id
    }

    /// 이 좌표에서 길을 막고 있는 엔티티
    pub fn blocker_at(&self, p: Point) -> Option<EntityId> {
        self.positions
            .iter()
            .find(|(id, pos)| pos.0 == p && self.blockers.get(*id).is_some())
            .map(|(id, _)| id)
    }
}

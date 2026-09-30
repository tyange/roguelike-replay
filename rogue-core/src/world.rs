use crate::comps::{BlocksTile, Glyph, Health, Monster, Name, Player, Position, Rgb, Target};
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
    pub targets: Store<Target>,
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
        self.targets.remove(id);
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
        self.healths.insert(id, Health { hp: 20, max: 20 });
        self.names.insert(id, Name("You".to_string()));
        self.blockers.insert(id, BlocksTile);
        self.players.insert(id, Player);
        id
    }

    /// 고블린: 'g', 초록(80, 200, 80), 체력 6, 이름 "Goblin", 길을 막고, Monster 표시, target 을 쫓는다
    pub fn spawn_goblin(&mut self, pos: Point, target: EntityId) -> EntityId {
        let id = self.entities.spawn();
        self.positions.insert(id, Position(pos));
        self.glyphs.insert(
            id,
            Glyph {
                ch: 'g',
                color: Rgb {
                    r: 80,
                    g: 200,
                    b: 80,
                },
            },
        );
        self.healths.insert(id, Health { hp: 6, max: 6 });
        self.names.insert(id, Name("Goblin".to_string()));
        self.blockers.insert(id, BlocksTile);
        self.monsters.insert(id, Monster);
        self.targets.insert(id, Target(target));
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

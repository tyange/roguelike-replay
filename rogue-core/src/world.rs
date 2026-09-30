use crate::comps::{
    Ai, BlocksTile, Combat, Glyph, Health, InBackpack, Item, Monster, Name, Player, Position, Rgb,
    Viewshed,
};
use crate::defs::{ItemDef, MonsterDef};
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
    pub viewsheds: Store<Viewshed>,
    pub items: Store<Item>,
    pub backpacks: Store<InBackpack>,
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
        self.viewsheds.remove(id);
        self.items.remove(id);
        self.backpacks.remove(id);
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
        self.viewsheds.insert(id, Viewshed::new(8));
        id
    }

    pub fn spawn_monster(&mut self, def: &MonsterDef, pos: Point) -> EntityId {
        let id = self.entities.spawn();
        self.positions.insert(id, Position(pos));
        self.glyphs.insert(
            id,
            Glyph {
                ch: def.glyph,
                color: def.color,
            },
        );
        self.healths.insert(
            id,
            Health {
                hp: def.hp,
                max: def.hp,
            },
        );
        self.combats.insert(
            id,
            Combat {
                power: def.power,
                defense: def.defense,
            },
        );
        self.names.insert(id, Name(def.name.clone())); // 정의는 계속 쓰이므로 복사한다
        self.blockers.insert(id, BlocksTile);
        self.monsters.insert(id, Monster);
        self.ais.insert(id, def.ai);
        self.viewsheds.insert(id, Viewshed::new(def.vision));
        id
    }

    /// 바닥에 놓인 아이템. 길을 막지 않으므로 BlocksTile 은 없다
    pub fn spawn_item(&mut self, def: &ItemDef, pos: Point) -> EntityId {
        let id = self.entities.spawn();
        self.positions.insert(id, Position(pos));
        self.glyphs.insert(
            id,
            Glyph {
                ch: def.glyph,
                color: def.color,
            },
        );
        self.names.insert(id, Name(def.name.clone())); // 정의는 계속 쓰이므로 복사한다
        self.items.insert(id, Item(def.effect));
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

/// 테스트에서만 쓰는 편의 함수. 정의는 이 프로젝트의 데이터 파일에서 꺼낸다
#[cfg(test)]
impl World {
    pub fn spawn_goblin(&mut self, pos: Point) -> EntityId {
        self.spawn_monster(&crate::defs::builtin().monsters.remove(0), pos)
    }

    pub fn spawn_kobold(&mut self, pos: Point) -> EntityId {
        self.spawn_monster(&crate::defs::builtin().monsters.remove(1), pos)
    }

    pub fn spawn_potion(&mut self, pos: Point) -> EntityId {
        self.spawn_item(&crate::defs::builtin().items.remove(0), pos)
    }

    pub fn spawn_scroll(&mut self, pos: Point) -> EntityId {
        self.spawn_item(&crate::defs::builtin().items.remove(1), pos)
    }
}

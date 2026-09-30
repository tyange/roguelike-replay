use crate::geom::Point;

/// 슬롯 번호와 세대 번호. 아무것도 빌리지 않는 그냥 숫자 두 개라서 어디에 저장해도 수명 문제가 없다
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityId {
    slot: u32,
    generation: u32,
}

#[derive(Debug, Clone)]
pub struct Creature {
    pub pos: Point,
    pub hp: i32,
    pub glyph: char,
    pub target: Option<EntityId>,
}

struct Slot {
    generation: u32,
    value: Option<Creature>,
}

pub struct Arena {
    slots: Vec<Slot>,
    free: Vec<u32>,
}

impl Arena {
    pub fn new() -> Arena {
        Arena {
            slots: Vec::new(),
            free: Vec::new(),
        }
    }

    /// 빈 슬롯이 있으면 재사용하고, 없으면 끝에 추가한다
    pub fn spawn(&mut self, creature: Creature) -> EntityId {
        if let Some(slot) = self.free.pop() {
            // 비어 있던 슬롯 재사용. 세대 번호는 despawn 때 이미 올라가 있다
            if let Some(s) = self.slots.get_mut(slot as usize) {
                s.value = Some(creature);
                return EntityId {
                    slot,
                    generation: s.generation,
                };
            }
        }
        let slot = u32::try_from(self.slots.len()).expect("슬롯 수가 u32 범위를 넘지 않는다");
        self.slots.push(Slot {
            generation: 0,
            value: Some(creature),
        });
        EntityId {
            slot,
            generation: 0,
        }
    }

    /// 슬롯을 비우고 세대를 올린다. 세대가 다른 ID 면 None
    pub fn despawn(&mut self, id: EntityId) -> Option<Creature> {
        let slot = self.slots.get_mut(id.slot as usize)?;
        if slot.generation != id.generation {
            return None; // 이미 다른 개체가 쓰고 있는 슬롯
        }
        let creature = slot.value.take()?;
        slot.generation += 1;
        self.free.push(id.slot);
        Some(creature)
    }

    /// 슬롯에 값이 있고 세대가 일치할 때만 Some
    pub fn get(&self, id: EntityId) -> Option<&Creature> {
        let slot = self.slots.get(id.slot as usize)?;
        if slot.generation == id.generation {
            slot.value.as_ref()
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, id: EntityId) -> Option<&mut Creature> {
        let slot = self.slots.get_mut(id.slot as usize)?;
        if slot.generation == id.generation {
            slot.value.as_mut()
        } else {
            None
        }
    }

    /// 살아 있는 모든 개체의 ID (2-3에서 이터레이터로 바꾼다)
    pub fn ids(&self) -> Vec<EntityId> {
        self.slots
            .iter()
            .enumerate()
            .filter(|(_, s)| s.value.is_some())
            .filter_map(|(i, s)| {
                Some(EntityId {
                    slot: u32::try_from(i).ok()?,
                    generation: s.generation,
                })
            })
            .collect()
    }
}

impl Default for Arena {
    fn default() -> Self {
        Arena::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn goblin() -> Creature {
        Creature {
            pos: Point::new(1, 1),
            hp: 5,
            glyph: 'g',
            target: None,
        }
    }

    #[test]
    fn despawned_id_returns_none() {
        let mut arena = Arena::new();
        let id = arena.spawn(goblin());
        assert!(arena.get(id).is_some());
        arena.despawn(id);
        assert!(arena.get(id).is_none());
        assert!(arena.get_mut(id).is_none());
    }

    #[test]
    fn stale_id_does_not_see_new_occupant() {
        let mut arena = Arena::new();
        let a = arena.spawn(goblin());
        arena.despawn(a);
        let b = arena.spawn(goblin()); // 같은 슬롯을 재사용한다
        assert_ne!(a, b);
        assert!(arena.get(a).is_none()); // 세대가 다르므로 B가 보이지 않는다
        assert!(arena.get(b).is_some());
        assert!(arena.despawn(a).is_none()); // A의 ID로 B를 죽일 수도 없다
        assert!(arena.get(b).is_some());
    }

    #[test]
    fn ids_lists_only_living() {
        let mut arena = Arena::new();
        let a = arena.spawn(goblin());
        let b = arena.spawn(goblin());
        arena.despawn(a);
        assert_eq!(arena.ids(), vec![b]);
    }
}

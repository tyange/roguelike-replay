use serde::{Deserialize, Serialize};

/// 슬롯 번호와 세대 번호. 아무것도 빌리지 않는 그냥 숫자 두 개
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityId {
    slot: u32,
    generation: u32,
}

impl EntityId {
    pub(crate) fn new(slot: u32, generation: u32) -> EntityId {
        EntityId { slot, generation }
    }

    pub(crate) fn index(self) -> usize {
        self.slot as usize
    }

    pub(crate) fn generation(self) -> u32 {
        self.generation
    }
}

#[derive(Serialize, Deserialize)]
struct Slot {
    generation: u32,
    alive: bool,
}

/// 엔티티가 "존재하는가"만 관리한다. 데이터는 컴포넌트 저장소가 가진다
#[derive(Default, Serialize, Deserialize)]
pub struct Entities {
    slots: Vec<Slot>,
    free: Vec<u32>,
}

impl Entities {
    pub fn spawn(&mut self) -> EntityId {
        if let Some(slot) = self.free.pop()
            && let Some(s) = self.slots.get_mut(slot as usize)
        {
            s.alive = true;
            return EntityId::new(slot, s.generation);
        }
        let slot = u32::try_from(self.slots.len()).expect("슬롯 수가 u32 범위를 넘지 않는다");
        self.slots.push(Slot {
            generation: 0,
            alive: true,
        });
        EntityId::new(slot, 0)
    }

    pub fn despawn(&mut self, id: EntityId) -> bool {
        match self.slots.get_mut(id.index()) {
            Some(s) if s.alive && s.generation == id.generation() => {
                s.alive = false;
                s.generation += 1;
                self.free.push(id.slot);
                true
            }
            _ => false,
        }
    }

    pub fn is_alive(&self, id: EntityId) -> bool {
        self.slots
            .get(id.index())
            .is_some_and(|s| s.alive && s.generation == id.generation())
    }

    /// 살아 있는 엔티티의 ID 들. 반환된 이터레이터는 self 를 빌린다
    pub fn iter(&self) -> impl Iterator<Item = EntityId> + '_ {
        self.slots.iter().enumerate().filter_map(|(i, s)| {
            if s.alive {
                Some(EntityId::new(u32::try_from(i).ok()?, s.generation))
            } else {
                None
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stale_id_is_not_alive_after_slot_reuse() {
        let mut entities = Entities::default();
        let a = entities.spawn();
        assert!(entities.despawn(a));
        let b = entities.spawn(); // 같은 슬롯
        assert_ne!(a, b);
        assert!(!entities.is_alive(a));
        assert!(entities.is_alive(b));
        assert!(!entities.despawn(a)); // A의 ID로 B를 지울 수 없다
        assert!(entities.is_alive(b));
    }

    #[test]
    fn iter_lists_living_only() {
        let mut entities = Entities::default();
        let a = entities.spawn();
        let b = entities.spawn();
        entities.despawn(a);
        assert_eq!(entities.iter().collect::<Vec<_>>(), vec![b]);
    }
}

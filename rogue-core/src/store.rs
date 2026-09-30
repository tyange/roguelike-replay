use crate::entity::EntityId;

/// 컴포넌트 하나를 담는 저장소. 슬롯 번호로 인덱싱되는 Vec이고, 각 칸은 (세대, 값) 또는 비어 있음
pub struct Store<T> {
    slots: Vec<Option<(u32, T)>>,
}

impl<T> Default for Store<T> {
    fn default() -> Self {
        Store { slots: Vec::new() }
    }
}

impl<T> Store<T> {
    pub fn insert(&mut self, id: EntityId, value: T) {
        let i = id.index();
        if self.slots.len() <= i {
            self.slots.resize_with(i + 1, || None);
        }
        if let Some(slot) = self.slots.get_mut(i) {
            *slot = Some((id.generation(), value));
        }
    }

    pub fn get(&self, id: EntityId) -> Option<&T> {
        match self.slots.get(id.index())? {
            Some((generation, value)) if *generation == id.generation() => Some(value),
            _ => None,
        }
    }

    pub fn get_mut(&mut self, id: EntityId) -> Option<&mut T> {
        match self.slots.get_mut(id.index())? {
            Some((generation, value)) if *generation == id.generation() => Some(value),
            _ => None,
        }
    }

    pub fn remove(&mut self, id: EntityId) -> Option<T> {
        let slot = self.slots.get_mut(id.index())?;
        match slot {
            Some((generation, _)) if *generation == id.generation() => {
                slot.take().map(|(_, value)| value)
            }
            _ => None,
        }
    }

    /// 살아 있는 모든 (ID, 값) 쌍. 반환된 이터레이터는 이 저장소를 빌리고 있다
    pub fn iter(&self) -> impl Iterator<Item = (EntityId, &T)> + '_ {
        self.slots.iter().enumerate().filter_map(|(i, slot)| {
            let (generation, value) = slot.as_ref()?;
            Some((EntityId::new(u32::try_from(i).ok()?, *generation), value))
        })
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (EntityId, &mut T)> + '_ {
        self.slots.iter_mut().enumerate().filter_map(|(i, slot)| {
            let (generation, value) = slot.as_mut()?;
            Some((EntityId::new(u32::try_from(i).ok()?, *generation), value))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::Entities;

    #[test]
    fn insert_get_remove() {
        let mut entities = Entities::default();
        let a = entities.spawn();
        let mut store = Store::default();
        store.insert(a, 10);
        assert_eq!(store.get(a), Some(&10));
        *store.get_mut(a).expect("방금 넣었다") += 1;
        assert_eq!(store.remove(a), Some(11));
        assert_eq!(store.get(a), None);
    }

    #[test]
    fn stale_id_gets_nothing() {
        let mut entities = Entities::default();
        let a = entities.spawn();
        let mut store = Store::default();
        store.insert(a, "old");
        entities.despawn(a);
        let b = entities.spawn(); // 같은 슬롯, 다른 세대
        store.insert(b, "new");
        assert_eq!(store.get(a), None);
        assert_eq!(store.get(b), Some(&"new"));
    }

    #[test]
    fn iter_skips_empty_slots() {
        let mut entities = Entities::default();
        let ids: Vec<_> = (0..4).map(|_| entities.spawn()).collect();
        let mut store = Store::default();
        store.insert(ids[1], 'b');
        store.insert(ids[3], 'd');
        let seen: Vec<_> = store.iter().map(|(_, v)| *v).collect();
        assert_eq!(seen, vec!['b', 'd']);
    }
}

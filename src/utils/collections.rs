use std::{cell::RefCell, borrow::BorrowMut, borrow::Borrow, collections::BTreeMap, pin::Pin, ops::DerefMut};

pub struct FrozenBTreeMap<K,V>(
    RefCell<BTreeMap<K, Pin<Box<V>>>>
);

impl<K,V> Default for FrozenBTreeMap<K,V> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<K,V> FrozenBTreeMap<K, V> 
where K: std::cmp::Ord
{
    pub fn insert(&self, key: K, v: V) {
        let pinned = Box::pin(v);
        self.0.borrow_mut().insert(key, pinned);
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.0.borrow().contains_key(key)
    }

    pub fn get(&mut self, key: &K) -> Option<Pin<&V>> {
        self.0.borrow()
        .get(&key)
        .as_deref()
        .map(|p| p.as_ref())
    }

    pub fn get_mut(&mut self, key: &K) -> Option<Pin<&mut V>> {
        self.0.borrow_mut()
        .deref_mut()
        .get_mut(&key)
        .as_deref_mut()
        .map(|p| p.as_mut())
    }
}
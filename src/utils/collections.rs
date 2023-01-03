use std::{collections::BTreeMap, pin::Pin, rc::Rc, cell::RefCell};

type FrozenValue<V> = Rc<RefCell<Pin<Box<V>>>>;
type BaseFrozenBTreeMap<K,V> = RefCell<BTreeMap<K, FrozenValue<V>>>;

pub struct FrozenBTreeMap<K,V>(
    BaseFrozenBTreeMap<K,V>
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
        let frozen = Rc::new(RefCell::new(Box::pin(v)));
        self.0.borrow_mut().insert(key, frozen);
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.0.borrow().contains_key(key)
    }

    pub fn get(&self, key: &K) -> Option<FrozenValue<V>> {
        self.0
        .borrow()
        .get(&key)
        .cloned()
    }
}
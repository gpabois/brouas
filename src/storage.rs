use self::traits::Storage as TraitStorage;

pub mod traits {
    pub trait Storage {
        type Key;
        type Value;
    
        fn fetch(&self, key: &Self::Key) -> Option<Self::Value>;
        fn store(&mut self, key: impl Into<Self::Key>, value: impl Into<Self::Value>);
    }
}

pub struct InMemory<K,V> {
    map: std::collections::HashMap<K,V>
}

impl<K: Eq + std::hash::Hash, V: Clone> TraitStorage for InMemory<K, V>
{
    type Key = K;
    type Value = V;

    fn fetch(&self, key: &Self::Key) -> Option<Self::Value> {
        self.map.get(key).cloned()
    }

    fn store(&mut self, key: impl Into<Self::Key>, value: impl Into<Self::Value>) {
        self.map.insert(key.into(), value.into());
    }   
}

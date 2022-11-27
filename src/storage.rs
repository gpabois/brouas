pub trait Storage {
    type Key;
    type Value;

    fn load(&self, key: &Self::Key) -> Option<Self::Value>;
    fn save(&mut self, key: impl Into<Self::Key>, value: impl Into<Self::Value>);
}

pub struct HashMap<K,V> {
    map: std::collections::HashMap<K,V>
}

impl<K: Eq + std::hash::Hash, V: Clone> Storage for HashMap<K, V>
{
    type Key = K;
    type Value = V;

    fn load(&self, key: &Self::Key) -> Option<Self::Value> {
        self.map.get(key).cloned()
    }

    fn save(&mut self, key: impl Into<Self::Key>, value: impl Into<Self::Value>) {
        self.map.insert(key.into(), value.into());
    }   
}

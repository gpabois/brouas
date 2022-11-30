use self::traits::Storage as TraitStorage;

pub mod traits {
    pub trait Storage {
        type Key: Clone + PartialEq;
        type Value;
    
        fn fetch(&self, key: &Self::Key) -> Option<Self::Value>;
        fn store(&mut self, key: impl Into<Self::Key>, value: impl Into<Self::Value>);
        fn store_all(&mut self, elements: impl Iterator<Item=(Self::Key, Self::Value)>);
    }

}

pub struct MutRefStorage<'a, Storage>(&'a mut Storage)
where Storage: self::traits::Storage;

impl<'a, Storage> From<&'a mut Storage> for MutRefStorage<'a, Storage>
where Storage: self::traits::Storage {
    fn from(store: &'a mut Storage) -> Self {
        Self(store)
    }
}

impl<'a, Storage> TraitStorage for MutRefStorage<'a, Storage>
where Storage: self::traits::Storage
{
    type Key = Storage::Key;
    type Value = Storage::Value;

    fn fetch(&self, key: &Self::Key) -> Option<Self::Value> {
        self.0.fetch(key)
    }

    fn store(&mut self, key: impl Into<Self::Key>, value: impl Into<Self::Value>) {
        self.0.store(key, value)
    }

    fn store_all(&mut self, elements: impl Iterator<Item=(Self::Key, Self::Value)>) {
        self.0.store_all(elements)
    }
}

pub struct InMemory<Key, Value> {
    map: std::collections::HashMap<Key, Value>
}

impl<Key: Eq + std::hash::Hash, Value: Clone> InMemory<Key, Value>
{
    pub fn new() -> Self {
        Self {
            map: Default::default()
        }
    }
}

impl<Key: Eq + std::hash::Hash + Clone, Value: Clone> TraitStorage for InMemory<Key, Value>
{
    type Key = Key;
    type Value = Value;

    fn fetch(&self, key: &Self::Key) -> Option<Self::Value> {
        self.map.get(key).cloned()
    }

    fn store(&mut self, key: impl Into<Self::Key>, value: impl Into<Self::Value>) {
        self.map.insert(key.into(), value.into());
    }

    fn store_all(&mut self, elements: impl Iterator<Item=(Self::Key, Self::Value)>) {
        elements.for_each(|(key, value)| self.store(key, value));
    } 
}

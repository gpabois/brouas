use self::traits::Storage as TraitStorage;

pub mod traits {
    pub trait Storage {
        type Key: Clone + PartialEq + 'static;
        type Value;
    
        fn fetch(&self, key: &Self::Key) -> Option<Self::Value>;
        fn fetch_all<'a>(&self, keys: impl Iterator<Item=&'a Self::Key>) -> Vec<(Self::Key, Self::Value)>;

        fn store(&mut self, key: impl Into<Self::Key>, value: impl Into<Self::Value>);
        fn store_all(&mut self, elements: impl Iterator<Item=(Self::Key, Self::Value)>);

        fn contains(&mut self, key: &Self::Key) -> bool;
    }
}

pub mod alg 
{
    pub struct Forwarder<'a, FromStorage, ToStorage> 
    where FromStorage: super::traits::Storage,
          ToStorage: super::traits::Storage<Key=FromStorage::Key, Value=FromStorage::Value>
    {
        from: &'a FromStorage,
        to: &'a mut ToStorage
    }

    impl<'a, FromStorage, ToStorage>  Forwarder<'a, FromStorage, ToStorage> 
    where FromStorage: super::traits::Storage,
          ToStorage: super::traits::Storage<Key=FromStorage::Key, Value=FromStorage::Value>
    {
        pub fn forward_values_if_not_present(&mut self, keys: impl Iterator<Item=FromStorage::Key>) -> Vec<FromStorage::Key>
        {
            let to_forward_keys: Vec<FromStorage::Key> = keys.filter(|key| !self.to.contains(key)).collect();
            
            self.to.store_all(
                self.from.fetch_all(
                    to_forward_keys.iter()
                ).into_iter()
            );

            to_forward_keys
        }
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

    fn fetch_all<'b>(&self, keys: impl Iterator<Item=&'b Self::Key>) -> Vec<(Self::Key, Self::Value)> {
        self.0.fetch_all(keys)
    }

    fn contains(&mut self, key: &Self::Key) -> bool {
        self.0.contains(key)
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

impl<Key: Eq + std::hash::Hash + Clone + 'static, Value: Clone> TraitStorage for InMemory<Key, Value>
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

    fn fetch_all<'a>(&self, keys: impl Iterator<Item=&'a Self::Key>) -> Vec<(Self::Key, Self::Value)> {
        keys
        .map(|key| (key, self.map.get(&key)))
        .filter(|(_, value)| value.is_some())
        .map(|(key, value)| (key.clone(), value.cloned().unwrap()))
        .collect()
    }

    fn contains(&mut self, key: &Self::Key) -> bool {
        self.map.contains_key(key)
    } 
}

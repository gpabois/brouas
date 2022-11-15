pub trait Storage {
    type Key;
    type Value;

    fn load(&self, key: impl AsRef<Self::Key>) -> Option<Self::Value>;
    fn save(&mut self, key: impl Into<Self::Key>, value: impl Into<Self::Value>);
}

pub struct HashMap<K,V> {
    map: std::collections::HashMap<K,V>
}

impl<K: Eq + std::hash::Hash, V: Clone> Storage for HashMap<K, V>
{
    type Key = K;
    type Value = V;

    fn load(&self, key: impl AsRef<Self::Key>) -> Option<Self::Value> {
        self.map.get(key.as_ref()).cloned()
    }

    fn save(&mut self, key: impl Into<Self::Key>, value: impl Into<Self::Value>) {
        self.map.insert(key.into(), value.into());
    }   
}

pub enum BrouasCell<Key, Hash> {
    Hash(Hash),
    Key(Key)
}
pub enum BrouasNode<const t: usize, Hash, Key, Element>
{
    Branch{
        hash: Hash, 
        children: Vec<BrouasCell<Hash, Key>>
    },
    Leaf {
        hash: Hash, 
        children: Vec<(Key, Element)>
    }
}

impl<const t: usize, Hash, Key, Element> BrouasNode<t, Hash, Key, Element>
{
    let prevCell: Option<BrouasCell<Key, Hash>> = None;
}


pub struct BrouasTree<
        const t: usize, 
        Store: Storage<
            Key=Hash, 
            Value=BrouasNode<t, Hash, Key, Element>
        >, 
        Key,
        Hash: Eq + std::hash::Hash, 
        Element: Clone
    >
{
    storage: Store,
    uncomitted: std::collections::HashMap<Hash, BrouasNode<t, Key, Hash, Element>>,
    root: Option<Hash>
}

pub type Path<const t: usize, Hash, Key, Element> = Vec<BrouasNode<t, Hash, Key, Element>>;

impl <const t: usize, 
    Store: Storage<
        Key=Hash, 
        Value=BrouasNode<t, Hash, Key, Element>
    >, 
    Key: PartialOrd + Ord,
    Hash: Eq + std::hash::Hash, 
    Element: Clone
> BrouasTree<t, Store, Key, Hash, Element> {

    pub fn get(&self, key: impl AsRef<Key>) -> Option<BrouasNode<t, Hash, Key, Element>>
    {
        
    }

    pub fn search(&self, key: impl AsRef<Key>) -> Path<t, Hash, Key, Element>
    {

    }
    
    pub fn search_node(&self, key: impl AsRef<Key>, node: &BrouasNode<t, Hash, Key, Element>) -> Option<Hash>
    {
        match node {
            BrouasNode::Branch { hash, children } => {
                children.iter().find(|p| {
                    match p {
                        BrouasCell::Hash(_) => false
                        BrouasCell::Key(k) => 
                    }
                });
            }
        }
    }
}
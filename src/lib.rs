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

pub enum BrouasBranchCell<Key, Hash> {
    Hash(Hash),
    Key(Key)
}

pub enum BrouasCell<'a, Hash, Key, Element> {
    Branch(usize, &'a BrouasBranchCell<Key, Hash>),
    Element(usize, &'a Element)
}
pub enum BrouasNode<const t: usize, Hash, Key, Element>
{
    Branch{
        hash: Hash, 
        children: Vec<BrouasBranchCell<Hash, Key>>
    },
    Leaf {
        hash: Hash, 
        children: Vec<(Key, Element)>
    }
}

impl<const t: usize, Hash, Key, Element> BrouasNode<t, Hash, Key, Element> {
    // Lower or equal for either branch cell or leaf element
    pub fn leq(&self, key: &Key) -> Option<usize> {
        match self {
            Self::Branch { hash, children } => {
                let iterator = children.iter().enumerate();
            },
            Self::Leaf { hash, children } => {

            }
        }
    }
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

#[derive(Default)]
pub struct Path<const t: usize, Hash, Key, Element>
{
    current: BrouasNode<t, Hash, Key, Element>,
    prev: Option<(usize, Path<t, Hash, Key, Element>)>
}

impl <const t: usize, Hash, Key, Element> Path<t, Hash, Key, Element>
{
    fn push(path: Self, node: BrouasNode<t, Hash, Key, Element>, index: usize) -> Self {
        Self {
            current: node,
            prev: Some((index, path))
        }
    }
}

impl <const t: usize, 
    Store: Storage<
        Key=Hash, 
        Value=BrouasNode<t, Hash, Key, Element>
    >, 
    Key: PartialOrd + Ord,
    Hash: Eq + std::hash::Hash, 
    Element: Clone
> BrouasTree<t, Store, Key, Hash, Element> 
{

    pub fn get_node(&self, hash: impl AsRef<Hash>) -> Option<BrouasNode<t, Hash, Key, Element>>
    {
        
    }

    pub fn search(&self, key: impl AsRef<Key>) -> Option<Path<t, Hash, Key, Element>>
    {
        if self.root.is_none() 
        {
            return None;
        }
        rec_search()
    }

    pub fn rec_search(&self, key: impl AsRef<Key>, hash: Hash, mut path: Path<t, Hash, Key, Element>, index: usize) -> Path<t, Hash, Key, Element>
    {
        if let Some(node) = self.get_node(hash) 
        {
            if let Some((cindex, chash)) = self.search_node(key, &node)
            {
                path = rec_search(key, chash, path.push(node, index), cindex)
            }
        } 

        path
    }
    
    pub fn search_node(&self, key: impl AsRef<Key>, node: &BrouasNode<t, Hash, Key, Element>) -> Option<(usize, Hash)>
    {
        match node {
            BrouasNode::Branch { hash, children } => {
                
            },
            _ => {}
        }
    }
}
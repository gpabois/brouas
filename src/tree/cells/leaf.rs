
#[derive(Clone)]
pub struct LeafCell<Key, Element>(Key, Element);

impl<Key, Element> LeafCell<Key, Element>
{
    pub fn new(key: Key, element: Element) -> Self {
        Self(key, element)
    }
 
}

impl<Key, Element> LeafCell<Key, Element>
where Key: Clone + crate::hash::traits::Hashable, Element: crate::hash::traits::Hashable + Clone
{
    pub fn update_hash<Hasher: crate::hash::traits::Hasher>(&self, hasher: &mut Hasher)
    {
        self.0.hash(hasher);
        self.1.hash(hasher);
    }
}

impl<Key: PartialOrd + PartialEq + Clone, Element: Clone> std::cmp::PartialOrd<Key> for LeafCell<Key, Element>
{
    fn partial_cmp(&self, other: &Key) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}

impl<Key: PartialOrd + PartialEq + Clone, Element: Clone> std::cmp::PartialOrd<&Key> for &mut LeafCell<Key, Element>
{
    fn partial_cmp(&self, other: &&Key) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}


impl<Key: PartialOrd + PartialEq + Clone, Element: Clone> std::cmp::PartialEq<Key> for LeafCell<Key, Element>
{
    fn eq(&self, other: &Key) -> bool {
        self.0 == *other
    }
}

pub mod traits {
    pub trait LeafCells
    {
        type Node: crate::tree::node::traits::Node;
    
        fn search<'a>(&'a self, k: &<Self::Node as crate::tree::node::traits::Node>::Key) -> Option<&'a <Self::Node as crate::tree::node::traits::Node>::Element>;
        fn search_mut<'a>(&'a mut self, k: &<Self::Node as crate::tree::node::traits::Node>::Key) -> Option<&'a mut <Self::Node as crate::tree::node::traits::Node>::Element>;
        
        fn split(&mut self) -> (<Self::Node as crate::tree::node::traits::Node>::Key, Self);
        fn is_full(&self) -> bool;
        fn insert(&mut self, key: <Self::Node as crate::tree::node::traits::Node>::Key, element: <Self::Node as crate::tree::node::traits::Node>::Element);
    
        fn compute_hash(&self) -> <Self::Node as crate::tree::node::traits::Node>::Hash;
    }
}

use self::traits::LeafCells as TraitLeafCells;
use crate::hash::traits::{Hash, Hasher};

#[derive(Clone)]
pub struct LeafCells<Node> 
where Node: crate::tree::node::traits::Node
{
    cells: Vec<LeafCell<Node::Key, Node::Element>>
}

impl<Node> LeafCells<Node>
where Node: crate::tree::node::traits::Node
{
    pub fn new(cell: LeafCell<Node::Key, Node::Element>) -> Self {
        Self{
            cells: vec![cell]
        }
    }
}

impl<Node> TraitLeafCells for LeafCells<Node>
where Node: crate::tree::node::traits::Node
{
    type Node = Node;

    fn search<'a>(&'a self, key: &Node::Key) -> Option<&'a Node::Element> {
        self.cells
        .iter()
        .find(|c| *c == key)
        .and_then(|c| Some(&(c.1)))
    }

    fn search_mut<'a>(&'a mut self, key: &<Self::Node as crate::tree::node::traits::Node>::Key) -> Option<&'a mut <Self::Node as crate::tree::node::traits::Node>::Element> {
        self.cells
        .iter_mut()
        .find(|c| *c == key)
        .and_then(|c| Some(&mut (c.1)))
    }

    fn split(&mut self) -> (Node::Key, Self)
    {
        let (left, right) = self.cells.split_at(Node::SIZE/2);
        let right_cells = Self {cells: right.iter().cloned().collect()};
        self.cells = left.iter().cloned().collect();
        (right_cells.cells[0].0.clone(), right_cells)
    }

    fn is_full(&self) -> bool {
        self.cells.len() >= Node::SIZE
    }

    fn insert(&mut self, key: Node::Key, element: Node::Element) {
        self.cells.push(LeafCell(key, element));
        self.cells.sort_unstable_by_key(|c| c.0.clone());
    }

    fn compute_hash(&self) -> <Self::Node as crate::tree::node::traits::Node>::Hash {
        let mut hasher = <Self::Node as crate::tree::node::traits::Node>::Hash::new_hasher();
        self.cells.iter().for_each(|cell| cell.update_hash(&mut hasher));
        hasher.finalize()
    }


}
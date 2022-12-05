
use crate::tree::node::traits::Node as TNode;

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
    use crate::tree::node::traits::Node as TNode;

    pub trait LeafCells<'a>
    {
        type Node: TNode<'a>;
    
        fn search(&'a self, k: &<Self::Node as TNode<'a>>::Key) -> Option<&'a <Self::Node as TNode<'a>>::Element>;
        fn search_mut(&'a mut self, k: &<Self::Node as TNode<'a>>::Key) -> Option<&'a mut <Self::Node as TNode<'a>>::Element>;
        
        fn split(&mut self) -> (Self, <Self::Node as TNode<'a>>::Key, Self) where Self: Sized;
        fn is_full(&self) -> bool;
        fn insert(&mut self, key: <Self::Node as TNode<'a>>::Key, element: <Self::Node as TNode<'a>>::Element);
    
        fn compute_hash(&self) -> <Self::Node as TNode<'a>>::Hash;
    }
}

use self::traits::LeafCells as TraitLeafCells;
use crate::hash::traits::{Hash, Hasher};

pub struct LeafCells<'a, Node> 
where Node: TNode<'a>
{
    cells: Vec<LeafCell<Node::Key, Node::Element>>
}

impl<'a, Node> LeafCells<'a, Node>
where Node: TNode<'a>
{
    pub fn new(cell: LeafCell<Node::Key, Node::Element>) -> Self {
        Self{
            cells: vec![cell]
        }
    }
}

impl<'a, Node> TraitLeafCells<'a> for LeafCells<'a, Node>
where Node: TNode<'a>
{
    type Node = Node;

    fn search(&'a self, key: &Node::Key) -> Option<&'a Node::Element> {
        self.cells
        .iter()
        .find(|c| *c == key)
        .and_then(|c| Some(&(c.1)))
    }

    fn search_mut(&'a mut self, key: &<Self::Node as TNode<'a>>::Key) -> Option<&'a mut <Self::Node as TNode<'a>>::Element> {
        self.cells
        .iter_mut()
        .find(|c| *c == key)
        .and_then(|c| Some(&mut (c.1)))
    }

    fn split(&mut self) -> (Self, Node::Key, Self)
    {
        let middle_index = Node::SIZE/2;

        let lefts: Vec<_> = self.cells.drain(0..middle_index).collect();
        let rights: Vec<_> = self.cells.drain(..).collect();
        let right_cells = Self {cells: rights};
        let left_cells = Self{cells: lefts};
        let middle_key = right_cells.cells[0].0.clone();

        return (left_cells, middle_key, right_cells)
    }

    fn is_full(&self) -> bool {
        self.cells.len() >= Node::SIZE
    }

    fn insert(&mut self, key: Node::Key, element: Node::Element) {
        self.cells.push(LeafCell(key, element));
        self.cells.sort_unstable_by_key(|c| c.0.clone());
    }

    fn compute_hash(&self) -> <Self::Node as TNode<'a>>::Hash {
        let mut hasher = <Self::Node as TNode>::Hash::new_hasher();
        self.cells.iter().for_each(|cell| cell.update_hash(&mut hasher));
        hasher.finalize()
    }


}
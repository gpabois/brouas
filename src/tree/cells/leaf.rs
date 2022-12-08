
use crate::tree::{node::traits::Node as TNode};

pub struct LeafCell<Key, Element>(Key, Element);

impl<Key,Element> ToOwned for LeafCell<Key, Element>
where Key: Clone, Element: ToOwned<Owned=Element>
{
    type Owned = Self;

    fn to_owned(&self) -> Self::Owned {
        Self(self.0.clone(), self.1.to_owned())
    }
}

impl<Key, Element> LeafCell<Key, Element>
{
    pub fn new(key: Key, element: Element) -> Self {
        Self(key, element)
    }
 
}

impl<Key, Element> LeafCell<Key, Element>
where Key: crate::hash::traits::Hashable, Element: crate::hash::traits::Hashable
{
    pub fn update_hash<Hasher: crate::hash::traits::Hasher>(&self, hasher: &mut Hasher)
    {
        self.0.hash(hasher);
        self.1.hash(hasher);
    }
}

impl<Key, Element> PartialOrd<Key> for LeafCell<Key, Element>
where Key: PartialOrd
{
    fn partial_cmp(&self, other: &Key) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}

impl<Key, Element> PartialOrd<&Key> for &mut LeafCell<Key, Element>
where Key: PartialOrd
{
    fn partial_cmp(&self, other: &&Key) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}


impl<Key, Element> PartialEq<Key> for LeafCell<Key, Element>
where Key: PartialOrd,
{
    fn eq(&self, other: &Key) -> bool {
        self.0 == *other
    }
}

pub mod traits {
    use crate::tree::node::traits::Node as TNode;

    pub trait LeafCells
    {
        type Node: TNode;
    
        fn search(&self, k: &<Self::Node as TNode>::Key) -> Option<&<Self::Node as TNode>::Element>;
        fn search_mut(&mut self, k: &<Self::Node as TNode>::Key) -> Option<&mut <Self::Node as TNode>::Element>;
        
        fn split(&mut self) -> (Self, <Self::Node as TNode>::Key, Self) where Self: Sized;
        fn is_full(&self) -> bool;
        fn insert(&mut self, key: <Self::Node as TNode>::Key, element: <Self::Node as TNode>::Element);
    
        fn compute_hash(&self) -> <Self::Node as TNode>::Hash;
    }
}

use self::traits::LeafCells as TraitLeafCells;
use crate::hash::traits::{Hash, Hasher};

pub struct LeafCells< Node> 
where Node: TNode
{
    cells: Vec<LeafCell<Node::Key, Node::Element>>
}

impl< Node> LeafCells< Node>
where Node: TNode
{
    pub fn new(cell: LeafCell<Node::Key, Node::Element>) -> Self {
        Self{
            cells: vec![cell]
        }
    }
}

impl<Node> ToOwned for LeafCells<Node>
where Node: TNode
{
    type Owned = Self;

    fn to_owned(&self) -> Self::Owned {
        Self {
            cells: self.cells.iter().map(|w| w.to_owned()).collect()
        }
    }
}

impl< Node> TraitLeafCells for LeafCells< Node>
where Node: TNode
{
    type Node = Node;

    fn search(&self, key: &Node::Key) -> Option<&Node::Element> {
        self.cells
        .iter()
        .find(|c| *c == key)
        .and_then(|c| Some(&(c.1)))
    }

    fn search_mut(&mut self, key: &<Self::Node as TNode>::Key) -> Option<&mut <Self::Node as TNode>::Element> {
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

    fn compute_hash(&self) -> <Self::Node as TNode>::Hash {
        let mut hasher = <Self::Node as TNode>::Hash::new_hasher();
        self.cells.iter().for_each(|cell| cell.update_hash(&mut hasher));
        hasher.finalize()
    }


}

#[derive(Clone)]
pub struct LeafCell<Key: Clone, Element: Clone>(Key, Element);

impl<Key: Clone, Element: Clone> LeafCell<Key, Element>
{
    pub fn new(key: Key, element: Element) -> Self {
        Self(key, element)
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
        fn split(&mut self) -> (<Self::Node as crate::tree::node::traits::Node>::Key, Self);
        fn is_full(&self) -> bool;
        fn insert(&mut self, key: <Self::Node as crate::tree::node::traits::Node>::Key, element: <Self::Node as crate::tree::node::traits::Node>::Element);
    }
}

use self::traits::LeafCells as TraitLeafCells;

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
}
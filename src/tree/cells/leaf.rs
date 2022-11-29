
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
    pub trait LeafCells<const SIZE: usize>
    {
        type Key: PartialEq + PartialOrd;
        type Element;
    
        fn search<'a>(&'a self, k: &Self::Key) -> Option<&'a Self::Element>;
        fn split(&mut self) -> (Self::Key, Self);
        fn is_full(&self) -> bool;
        fn insert(&mut self, key: Self::Key, element: Self::Element);
    }
}

use self::traits::LeafCells as TraitLeafCells;

pub struct LeafCells<const SIZE: usize, Key: PartialEq + PartialOrd + Clone, Element: Clone> 
{
    cells: Vec<LeafCell<Key, Element>>
}

impl<const SIZE: usize, Key: PartialEq + Ord + Clone, Element: Clone> LeafCells<SIZE, Key, Element>
{
    pub fn new(cell: LeafCell<Key, Element>) -> Self {
        Self{
            cells: vec![cell]
        }
    }
}

impl<const SIZE: usize, Key: PartialEq + Ord + Clone, Element: Clone> TraitLeafCells<SIZE> for LeafCells<SIZE, Key, Element>
{
    type Key = Key;
    type Element = Element;

    fn search<'a>(&'a self, key: &Self::Key) -> Option<&'a Self::Element> {
        self.cells
        .iter()
        .find(|c| *c == key)
        .and_then(|c| Some(&(c.1)))
    }

    fn split(&mut self) -> (Self::Key, Self)
    {
        let (left, right) = self.cells.split_at(SIZE/2);
        let right_cells = Self {cells: right.iter().cloned().collect()};
        self.cells = left.iter().cloned().collect();
        (right_cells.cells[0].0.clone(), right_cells)
    }

    fn is_full(&self) -> bool {
        self.cells.len() >= SIZE
    }

    fn insert(&mut self, key: Self::Key, element: Self::Element) {
        self.cells.push(LeafCell(key, element));
        self.cells.sort_unstable_by_key(|c| c.0.clone());
    }
}
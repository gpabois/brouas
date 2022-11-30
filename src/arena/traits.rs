pub trait Allocator<ElementRef, Element>
{
    fn allocate(&mut self, element: Element) -> ElementRef;
}

pub trait Arena: Allocator<Self::ElementRef, Self::Value>
{
    type ElementRef;
    type Value;

    fn borrow_element<'a>(&'a self, node_ref: &Self::ElementRef) -> Option<&'a Self::Value>;
    fn borrow_mut_element<'a>(&'a mut self, index: &Self::ElementRef) -> Option<&'a mut Self::Value>;

    fn contains(&mut self, element_ref: &Self::ElementRef) -> bool;
}


/// Two level element ref
pub trait TLElementRef
{
    type LocalIndex;
    type ForeignIndex;

    fn is_loaded(&self) -> bool;
    fn unload(&self, foreign_index: Self::ForeignIndex);
    fn load(&self, local_index: Self::LocalIndex);
    
    fn from_foreign_index(foreign_index: Self::ForeignIndex) -> Self;
    fn from_local_index(index: Self::LocalIndex) -> Self;

    fn get_local_index(&self) -> Option<Self::LocalIndex>;
    fn get_foreign_index(&self) -> Option<Self::ForeignIndex>;
}
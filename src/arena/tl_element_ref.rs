use std::{cell::RefCell};

use super::local_index::LocalIndex;
use super::traits::TLElementRef as TraitTLElementRef;

#[derive(Clone, PartialEq)]
struct BareTLElementRef<ForeignKey>(
    Option<LocalIndex>,
    Option<ForeignKey>
);

impl<ForeignKey> BareTLElementRef<ForeignKey>
{
    pub fn new(local: Option<LocalIndex>, foreign: Option<ForeignKey>) -> Self
    {
        Self(local, foreign)
    }
}

#[derive(Clone, PartialEq)]
pub struct TLElementRef<ForeignIndex: Clone + PartialEq>(RefCell<BareTLElementRef<ForeignIndex>>);

impl<ForeignIndex: Clone + PartialEq> TraitTLElementRef for TLElementRef<ForeignIndex>
{
    type ForeignIndex = ForeignIndex;
    type LocalIndex = LocalIndex;

    fn is_loaded(&self) -> bool {
        self.0.borrow().0.is_some()
    }

    fn unload(&self, foreign_index: Self::ForeignIndex) {
        self.0.borrow_mut().0 = None;
        self.0.borrow_mut().1 = Some(foreign_index);
    }

    fn load(&self, local_index: Self::LocalIndex) {
        self.0.borrow_mut().0 = Some(local_index);
        self.0.borrow_mut().1 = None;
    }

    fn from_foreign_index(foreign: Self::ForeignIndex) -> Self {
        Self(RefCell::new(BareTLElementRef::new(None, Some(foreign))))
    }

    fn from_local_index(local: Self::LocalIndex) -> Self {
        Self(RefCell::new(BareTLElementRef::new(Some(local), None)))
    }

    fn get_local_index(&self) -> Option<Self::LocalIndex> {
        self.0.borrow().0.clone()
    }

    fn get_foreign_index(&self) -> Option<Self::ForeignIndex> {
        self.0.borrow().1.clone()
    }
}
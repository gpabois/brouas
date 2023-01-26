use std::ops::{Deref, DerefMut};

#[derive(Default)]
pub struct Counter(std::cell::RefCell<u64>);

impl Counter {
    pub fn inc(&self) -> u64 {
        *self.0.borrow_mut().deref_mut() += 1;
        *self.0.borrow_mut()
    }
}

pub mod traits {
    pub trait ResetableIterator: Iterator {
        fn reset(&mut self);
    }

    pub trait CursorIterator: Iterator {
        fn current(&self) -> Self::Item;
    }
}
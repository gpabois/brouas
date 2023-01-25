use std::ops::{Deref, DerefMut};

#[derive(Default)]
pub struct Counter(std::cell::RefCell<u64>);

impl Counter {
    pub fn inc(&self) -> u64 {
        *self.0.borrow_mut().deref_mut() += 1;
        *self.0.borrow_mut()
    }
}

pub struct Watcher<T>{
    value: T,
    modified: bool
}

impl<T> Watcher<T> {
    pub fn new(value: T) -> Self {
        Self {
            value: value,
            modified: true
        }
    }

    pub fn wrap(value: T) -> Self {
        Self {
            value: value,
            modified: false
        }
    }

    pub fn is_modified(&self) -> bool {
        self.modified
    }

    pub fn done(&mut self)  {
        self.modified = false;
    }
}

impl<T> Deref for Watcher<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> DerefMut for Watcher<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.modified = true;
        &mut self.value
    }
}
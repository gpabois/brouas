use std::ops::{Deref, DerefMut};

pub mod collections;

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
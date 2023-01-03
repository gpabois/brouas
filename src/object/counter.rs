use std::{cell::RefCell, rc::Rc, ops::Deref};

use super::ObjectId;

#[derive(Clone, Default)]
pub struct SharedObjectCounter(Rc<RefCell<ObjectId>>);

impl SharedObjectCounter {
    pub fn new_id(&self) -> ObjectId {
        *self.0.deref().borrow_mut() += 1;
        *self.0.borrow()
    }
}


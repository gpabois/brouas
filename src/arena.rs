use std::ops::DerefMut;

use elsa::FrozenVec;

pub struct ArenaId(usize);
pub struct Arena<Element>(FrozenVec<Box<Element>>);

impl<Element> Arena<Element>
{
    pub fn new() -> Self {
        Self(FrozenVec::default())
    }
    pub fn alloc(&self, element: Element) -> ArenaId
    {
        self.0.push(Box::new(element));
        ArenaId(self.0.len() - 1)
    }

    pub fn upgrade(&self, id: &ArenaId) -> Option<&Element>
    {
        self.0.get(id.0)
    }

    pub fn upgrade_mut(&mut self, id: &ArenaId) -> Option<&mut Element>
    {
        Some(self.0.as_mut().get_mut(id.0).unwrap().deref_mut())
    }
}
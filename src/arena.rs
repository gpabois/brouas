use std::cell::{RefCell, UnsafeCell};
use std::marker::PhantomData;
use std::rc::Rc;
use std::collections::HashMap;

#[derive(Clone, PartialEq)]
pub struct ElementRef<Key: Clone + PartialEq>(RefCell<ElementState<Key>>);

impl<Key: Clone + PartialEq> ElementRef<Key>
{
    pub fn from_key(key: Key) -> Self {
        Self(RefCell::new(ElementState(None, Some(key))))
    }

    pub fn from_index(index: &ElementIndex) -> Self {
        Self(
            RefCell::new(
                ElementState(
                    Some(Rc::new(index.clone())), None
                )
            )
        )
    }

    pub fn get_index<'a>(&'a self) -> Option<Rc<ElementIndex>>
    {
        self.0.borrow().0.clone()
    }

    pub fn get_key<'a>(&'a self) -> Option<Key>
    {
       self.0.borrow().1.clone()
    }

    pub fn is_loaded(&self) -> bool {
        self.0.borrow().0.is_some()
    }

    pub fn has_key(&self) -> bool {
        self.0.borrow().1.is_some()
    }

    pub fn load(&self, index: Rc<ElementIndex>)
    {
        self.0.borrow_mut().0 = Some(index);
    }

    pub fn unload(&self, key: Key)
    {
        self.0.borrow_mut().1 = Some(key);
        self.0.borrow_mut().0 = None;
    }
}

#[derive(Clone, PartialEq)]
pub struct ElementState<Key>(
    Option<Rc<ElementIndex>>,
    Option<Key>
);

#[derive(Ord, Eq, PartialOrd, PartialEq, Default, Clone, Hash)]
pub struct ElementIndex(usize);

pub trait Allocator<Key: Clone + PartialEq, Element>
{
    fn allocate(&mut self, element: Element) -> ElementRef<Key>;
}

pub trait Arena
{
    type Key: Clone + PartialEq;
    type Value;

    fn borrow_element<'a>(&'a self, node_ref: &ElementRef<Self::Key>) -> Option<&'a Self::Value>;
    fn borrow_mut_element<'a>(&'a mut self, index: &ElementRef<Self::Key>) -> Option<&'a mut Self::Value>;

    fn allocate(&mut self, element: Self::Value) -> ElementRef<Self::Key>;
    fn contains(&mut self, node_ref: &ElementRef<Self::Key>) -> bool;

    fn load_if_not<Storage: crate::storage::Storage<Key=Self::Key, Value=Self::Value>>(
        &self, node_ref: &ElementRef<Self::Key>, storage: &Storage
    );

    /// Sweep the elements whose ElementRef is not shared anymore
    fn sweep(&mut self);
}

#[derive(Default)]
pub struct HashMapArena<Key, Value>
{
    _k: PhantomData<Key>,
    map: UnsafeCell<HashMap<ElementIndex, Value>>,
    last_index: UnsafeCell<ElementIndex>
}

impl<Key: PartialEq + Clone, Value> HashMapArena<Key, Value>
{
    fn inc_index(&self) -> ElementIndex
    {
        unsafe {
            self.last_index.get().as_mut().unwrap().0 += 1;
            return self.last_index.get().as_mut().unwrap().clone()
        }
    }
}

impl<Key: PartialEq + Clone, Value> Arena for HashMapArena<Key, Value>
{
    type Key = Key;
    type Value = Value;

    fn borrow_element<'a>(&'a self, node_ref: &ElementRef<Self::Key>) -> Option<&'a Self::Value> {
        if let Some(index) = node_ref.get_index() {
            unsafe {
                return self.map
                .get()
                .as_ref()
                .unwrap()
                .get(&index);
            }
        }

        return None
    }

    fn borrow_mut_element<'a>(&'a mut self, node_ref: &ElementRef<Self::Key>) -> Option<&'a mut Self::Value> {
        if let Some(index) = node_ref.get_index() {
            unsafe {
                return self.map
                .get()
                .as_mut()
                .unwrap()
                .get_mut(&index);
            }
        }

        return None
    }

    fn allocate(&mut self, element: Self::Value) -> ElementRef<Self::Key> {
        unsafe {
            let index = self.inc_index();
            let node_ref = ElementRef::from_index(&index);
            
            self.map
            .get()
            .as_mut()
            .unwrap()
            .insert(index, element);
    
            node_ref
        }
    }

    fn contains(&mut self, node_ref: &ElementRef<Self::Key>) -> bool {
        if let Some(index) = node_ref.get_index()
        {
            unsafe {
                return self.map.get()
                .as_ref()
                .unwrap()
                .contains_key(&index);
            }
        }

        return false;
    }

    fn load_if_not<Storage: crate::storage::Storage<Key=Self::Key, Value=Self::Value>>(
        &self, node_ref: &ElementRef<Self::Key>, storage: &Storage
    ) {
       if !node_ref.is_loaded() && node_ref.has_key() {
            unsafe 
            {
                if let Some(element) = storage.load(&node_ref.get_key().unwrap())
                {
                    let index = self.inc_index();

                    self.map
                    .get()
                    .as_mut()
                    .unwrap()
                    .insert(index.clone(), element);

                    node_ref.load(Rc::new(index));
                }
                
            }
       }
    }

    fn sweep(&mut self) {
        todo!()
    }
}
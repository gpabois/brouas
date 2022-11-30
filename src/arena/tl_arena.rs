use std::{cell::UnsafeCell, collections::HashMap};
use super::{local_index::LocalIndex, traits::{Arena, Allocator}, tl_element_ref::TLElementRef};
use super::traits::TLElementRef as TraitTLElementRef;

pub mod traits {
    use crate::arena::{traits::{Arena, TLElementRef}};
    use crate::storage::traits::Storage;

    pub trait TLArena : Arena
    {
        type Storage: Storage;
        type TLElementRef: TLElementRef<ForeignIndex = <Self::Storage as Storage>::Key>;

        fn save_elements(&mut self, elements: impl Iterator<Item=Self::TLElementRef>);
    }
}

mod alg {
    use std::collections::HashMap;

    pub fn inc_local_index<LocalIndex: std::ops::AddAssign<i32> + Copy>(index: &mut LocalIndex) -> LocalIndex
    {
        *index += 1;
        return *index;
    }

    pub fn save_elements<TLElementRef, Storage> (
            elements: impl Iterator<Item=TLElementRef>, 
            store: &mut Storage,
            map: &HashMap<TLElementRef::LocalIndex, Storage::Value>
        )
        where Storage: crate::storage::traits::Storage,
                <Storage as crate::storage::traits::Storage>::Value: Clone,
                TLElementRef: crate::arena::traits::TLElementRef<ForeignIndex = Storage::Key>,
                <TLElementRef as crate::arena::traits::TLElementRef>::LocalIndex: Clone + Eq + std::hash::Hash
                
     {
        let iter = elements
        .filter(|rf| rf.is_loaded())
        .map(|rf| {
            (
                rf.get_foreign_index().unwrap(), 
                map.get(&rf.get_local_index().unwrap()).cloned().unwrap()
            )
        });
        
        store.store_all(iter)
    }

    pub fn load_if_not<TLElementRef, Storage>(
        internal_index_counter: &mut TLElementRef::LocalIndex,
        element_ref: &TLElementRef, 
        store: &Storage, 
        map: &mut HashMap<TLElementRef::LocalIndex, Storage::Value>
    )
    where Storage: crate::storage::traits::Storage,
          TLElementRef: crate::arena::traits::TLElementRef<ForeignIndex = Storage::Key>,
          <TLElementRef as crate::arena::traits::TLElementRef>::LocalIndex: Clone + std::ops::AddAssign<i32> + Eq + std::hash::Hash + Copy
    {
        if !element_ref.is_loaded() {
            if let Some(element) = store
            .fetch(&element_ref.get_foreign_index().unwrap())
            {
                let local_index = inc_local_index(internal_index_counter);
                map.insert(local_index.clone(), element);
                element_ref.load(local_index);
            }   
       }
    }
}

#[derive(Default)]
pub struct TLArena<Storage>
where Storage: crate::storage::traits::Storage
{
    store: Storage,
    map: UnsafeCell<HashMap<LocalIndex, Storage::Value>>,
    last_index: UnsafeCell<LocalIndex>
}

impl<Storage> TLArena<Storage>
where Storage: crate::storage::traits::Storage
{
    pub fn new(storage: Storage) -> Self {
        Self {
            store: storage,
            map: UnsafeCell::default(),
            last_index: UnsafeCell::default()
        }
    }


    /// Increase local index counter
    fn inc_local_index(&self) -> LocalIndex {
        unsafe {
            self::alg::inc_local_index(self.last_index.get().as_mut().unwrap())
        }
    } 
}

impl<Storage> self::traits::TLArena for TLArena<Storage>
where Storage: crate::storage::traits::Storage,
      Storage::Value: Clone
{
    type Storage = Storage;
    type TLElementRef = TLElementRef<Storage::Key>;

    fn save_elements(&mut self, elements: impl Iterator<Item=Self::TLElementRef>) {
        unsafe {
            let ref_map = self.map.get().as_ref().unwrap();

            self::alg::save_elements(
                elements, 
                &mut self.store, 
                ref_map
            )
        }
    }
}

impl<Storage> TLArena<Storage>
where Storage: crate::storage::traits::Storage
{
    fn load_if_not(&self, element_ref: &TLElementRef<Storage::Key>) 
    {
        unsafe {
            self::alg::load_if_not(
                self.last_index.get().as_mut().unwrap(), 
                element_ref, 
                &self.store, 
                self.map.get().as_mut().unwrap()
            )
        }
    }
}

impl<Storage> Allocator<TLElementRef<Storage::Key>, Storage::Value> for TLArena<Storage>
where Storage: crate::storage::traits::Storage
{
    
    fn allocate(&mut self, element: Storage::Value) -> TLElementRef<Storage::Key> {
        unsafe {
            let index = self.inc_local_index();
            let element_ref = TLElementRef::from_local_index(index.clone());
            
            self.map
            .get()
            .as_mut()
            .unwrap()
            .insert(index, element);
    
            element_ref
        }
    }

}

impl<Storage> Arena for TLArena<Storage>
where Storage: crate::storage::traits::Storage
{
    type ElementRef = TLElementRef<Storage::Key>;
    type Value = Storage::Value;

    fn borrow_element<'a>(&'a self, element_ref: &Self::ElementRef) -> Option<&'a Self::Value> {
        self.load_if_not(element_ref);
        
        if let Some(index) = &element_ref.get_local_index() {
            unsafe {
                return self.map
                .get()
                .as_ref()
                .unwrap()
                .get(index);
            }
        }

        return None
    }

    fn borrow_mut_element<'a>(&'a mut self, element_ref: &Self::ElementRef) -> Option<&'a mut Self::Value> {
        self.load_if_not(element_ref);

        if let Some(index) = &element_ref.get_local_index() {
            unsafe {
                return self.map
                .get()
                .as_mut()
                .unwrap()
                .get_mut(index);
            }
        }

        return None
    }
    fn contains(&mut self, element_ref: &Self::ElementRef) -> bool {
        self.load_if_not(element_ref);
        if let Some(index) = &element_ref.get_local_index()
        {
            unsafe {
                return self.map.get()
                .as_ref()
                .unwrap()
                .contains_key(index);
            }
        }

        return false;
    }
}
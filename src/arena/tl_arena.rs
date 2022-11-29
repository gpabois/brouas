use std::{cell::UnsafeCell, collections::HashMap};
use super::{local_index::LocalIndex, traits::{Arena, Allocator}, tl_element_ref::TLElementRef};
use super::traits::TLElementRef as TraitTLElementRef;

#[derive(Default)]
pub struct TLArena<Key, Value, Storage>
where Storage: crate::storage::traits::Storage<Key=Key, Value=Value>
{
    store: Storage,
    map: UnsafeCell<HashMap<LocalIndex, Value>>,
    last_index: UnsafeCell<LocalIndex>
}

impl<Key: PartialEq + Clone, Value, Storage> TLArena<Key, Value, Storage>
where Key: PartialEq + Clone, 
      Storage: crate::storage::traits::Storage<Key=Key, Value=Value>
{
    fn inc_local_index(&self) -> LocalIndex
    {
        unsafe {
            self.last_index.get().as_mut().unwrap().0 += 1;
            return self.last_index.get().as_mut().unwrap().clone()
        }
    }
}

impl<Key, Value, Storage> TLArena<Key, Value, Storage>
where Key: PartialEq + Clone, 
      Storage: crate::storage::traits::Storage<Key=Key, Value=Value>
{
    fn load_if_not(&self, element_ref: &TLElementRef<Key>) {
       if !element_ref.is_loaded() {
            unsafe 
            {
                if let Some(element) = self.store
                .fetch(&element_ref.get_foreign_index().unwrap())
                {
                    let local_index = self.inc_local_index();

                    self.map
                    .get()
                    .as_mut()
                    .unwrap()
                    .insert(local_index.clone(), element);

                    element_ref.load(local_index);
                }   
            }
       }
    }
}

impl<Key: PartialEq + Clone, Value, Storage> Allocator<TLElementRef<Key>, Value> for TLArena<Key, Value, Storage>
where Key: PartialEq + Clone, 
      Storage: crate::storage::traits::Storage<Key=Key, Value=Value>
{
    
    fn allocate(&mut self, element: Value) -> TLElementRef<Key> {
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

impl<Key: PartialEq + Clone, Value, Storage> Arena for TLArena<Key, Value, Storage>
where Key: PartialEq + Clone, 
      Storage: crate::storage::traits::Storage<Key=Key, Value=Value>
{
    type ElementRef = TLElementRef<Key>;
    type Value = Value;

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
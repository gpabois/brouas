use std::{alloc::Layout, ops::{Deref, DerefMut}};

#[derive(Debug)]
pub enum Error {
    NotEnoughSpace
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct RawBufferCell<'buffer>
{
    block: *mut BufferBlock,
    _pht: std::marker::PhantomData<&'buffer ()>
}

impl<'buffer> From<*mut BufferBlock> for RawBufferCell<'buffer> {
    fn from(ptr: *mut BufferBlock) -> Self {
        Self::new(ptr)
    }
}

impl<'buffer> Clone for RawBufferCell<'buffer> {
    fn clone(&self) -> Self {
        Self::new(self.block)
    }
}

impl<'buffer> RawBufferCell<'buffer> {
    fn new(block: *mut BufferBlock) -> Self {
        unsafe {
            (*block).rc += 1;
        }

        Self {
            block,
            _pht: Default::default()
        }
    }

    pub fn leak(&self) -> *mut BufferBlock {
        self.block
    }

    pub fn leak_mut(&mut self) -> *mut BufferBlock {
        self.raise_modification_flag();
        self.block
    }

    pub fn rc(&self) -> usize {
        unsafe {
            (*self.block).rc
        }
    }

    pub fn raise_modification_flag(&mut self) {
        unsafe {
            (*self.block).modified = true;
        }       
    }
    /// Remove the modification flag
    pub fn drop_modification_flag(&mut self) {
        unsafe {
            (*self.block).modified = false;
        }
    }

    pub fn is_modified(&self) -> bool {
        unsafe {
            (*self.block).is_modified()
        }
    }

    pub fn try_into<T>(self) -> Option<BufferCell<'buffer, T>> {
        unsafe {
            let size = std::mem::size_of::<T>();
            let block_size = size.wrapping_div_euclid((*self.block).size);
            if size == block_size {
                return Some(BufferCell::new(self))
            } else {
                return None
            }
        }   
    }

    pub fn try_into_array<T>(self) -> Option<ArrayBufferCell<'buffer, T>> {
        unsafe {
            let size = std::mem::size_of::<T>();
            let block_size = (*self.block).size;

            let capacity = block_size.wrapping_div_euclid(size);
            let rem = block_size.wrapping_rem_euclid(size);

            if rem == 0 && capacity > 0 {
                return Some(ArrayBufferCell::new(self, capacity))
            } else {
                return None
            }
        }
    }
}

impl<'buffer> Drop for RawBufferCell<'buffer> {
    fn drop(&mut self) {
        unsafe {
            (*self.block).rc -= 1;
        }
    }
}

pub struct BufferCell<'buffer, T: ?Sized> {
    raw: RawBufferCell<'buffer>,
    _pht: std::marker::PhantomData<T>
}

impl<'a, T> Deref for BufferCell<'a, T> {
    type Target = T;

    fn deref(&self) -> &'a Self::Target {
        unsafe {
            BufferBlock::leak_value_unchecked::<T>(self.raw.leak()).as_ref().unwrap()
        }
    }
}

impl<'a, T> DerefMut for BufferCell<'a, T> {
    fn deref_mut(&mut self) -> &'a mut Self::Target {
        unsafe {
            BufferBlock::leak_value_unchecked::<T>(self.raw.leak_mut()).as_mut().unwrap()
        }
    }
}

impl<'buffer, T> BufferCell<'buffer, T> 
{
    fn new(block: impl Into<RawBufferCell<'buffer>>) -> Self {
        Self {
            raw: block.into(),
            _pht: Default::default()
        }
    }

}

impl<'a, T> Clone for BufferCell<'a, T> {
    fn clone(&self) -> Self {
        Self::new(self.raw.clone())
    }
}

pub struct ArrayBufferCell<'buffer, T> {
    len: usize,
    raw: RawBufferCell<'buffer>,
    _pht: std::marker::PhantomData<T>
}

impl<'buffer, T> ArrayBufferCell<'buffer, T> {
    fn new(raw: impl Into<RawBufferCell<'buffer>>, len: usize) -> Self {
        Self {
            len,
            raw: raw.into(),
            _pht: Default::default()
        }
    }

    pub fn is_modified(&self) -> bool {
        self.raw.is_modified()
    }

    pub fn drop_modification_flag(&mut self) {
        self.raw.drop_modification_flag()
    }
}

impl<'buffer, T> Deref for ArrayBufferCell<'buffer, T> {
    type Target = [T];

    fn deref(&self) -> &'buffer Self::Target {
        unsafe {
            std::slice::from_raw_parts(BufferBlock::leak_value_unchecked::<T>(self.raw.leak()), self.len)
        }
    }
}

impl<'a, T> DerefMut for ArrayBufferCell<'a, T> {
    fn deref_mut(&mut self) -> &'a mut Self::Target {
        unsafe {
            std::slice::from_raw_parts_mut(BufferBlock::leak_value_unchecked::<T>(self.raw.leak_mut()), self.len)
        }
    }
}
pub struct BufferBlock {
    pub size:       usize,
    pub free:       bool,
    pub rc:         usize,
    pub lru:        usize,
    pub modified:   bool,
    pub next:       *mut BufferBlock
}

impl BufferBlock {
    pub fn new(size: usize) -> Self {
        Self {
            size,
            free: false,
            rc: 0,
            lru: 0,
            modified: false,
            next: std::ptr::null_mut()
        }
    }

    pub fn size_of(size: usize) -> usize {
        std::mem::size_of::<Self>() + size
    }

    pub unsafe fn tail(raw: *mut Self) -> *mut Self {
        (raw as *mut u8).offset((Self::size_of((*raw).size)) as isize) as *mut Self
    }

    pub unsafe fn leak_value_unchecked<T>(raw: *mut Self) -> *mut T {
        (*raw).lru += 1;
        (raw as *mut u8).offset(std::mem::size_of::<Self>() as isize) as *mut T
    }

    pub fn match_size(&self, size: usize) -> bool {
        self.size == size
    }

    pub fn lru(&self) -> usize {
        return self.lru
    }

    pub fn is_modified(&self) -> bool {
        return self.modified
    }

    pub fn is_free(&self) -> bool {
        return self.free
    }

    pub fn is_unshared(&self) -> bool {
        self.rc == 0
    }
}

pub struct Buffer 
{
    pub layout: Layout,
    // The base of the allocated area
    pub base: *mut BufferBlock,
    // Last element of the linked list of blocks
    pub last: std::cell::RefCell<*mut BufferBlock>,
    // The tail of the managed area, and the head of the heap.
    pub tail: std::cell::RefCell<*mut BufferBlock>,
    // The limit of the whole allocated area
    pub end: *mut BufferBlock,
    // Number of allocated blocks in the buffer
    pub block_count: usize
}

struct BufferBlockIterator(*mut BufferBlock);

impl Iterator for BufferBlockIterator {
    type Item = *mut BufferBlock;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0.is_null() {
            return None;
        } else {
            let block = self.0;
            unsafe {
                self.0 = (*self.0).next;
            }
            return Some(block);
        }
    }
}

pub struct BufferCellIterator<'buffer>(BufferBlockIterator, std::marker::PhantomData<&'buffer ()>);

impl<'buffer> Iterator for BufferCellIterator<'buffer> {
    type Item = RawBufferCell<'buffer>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0.next() {
            None => None,
            Some(block) => Some(RawBufferCell::from(block))
        }
    }
}


impl Buffer {
    /// Create a buffer intented to be used with equal_sized memory blocks.
    pub fn new_by_type<T>(capacity: usize) -> Self {
        let align   = (std::mem::size_of::<T>() + std::mem::size_of::<BufferBlock>()).next_power_of_two();
        let size    = capacity.wrapping_mul(align);
        let layout = Layout::from_size_align(size, align).unwrap();
        
        unsafe {
            let base = std::alloc::alloc_zeroed(layout) as *mut BufferBlock;
            let tail = std::cell::RefCell::new(base);
            let end = (base as *mut u8).offset(size as isize) as *mut BufferBlock;    
            Self { layout, base, last: std::cell::RefCell::new(std::ptr::null_mut()), tail, end, block_count: 0 }       
        }
    }

    pub fn new_by_array<T>(array_size: usize, capacity: usize) -> Self {
        let block_data_size = std::mem::size_of::<T>().wrapping_mul(array_size);
        let align   = (block_data_size + std::mem::size_of::<BufferBlock>()).next_power_of_two();
        let size    = capacity.wrapping_mul(align);
        let layout = Layout::from_size_align(size, align).unwrap();
        
        unsafe {
            let base = std::alloc::alloc_zeroed(layout) as *mut BufferBlock;
            let tail = std::cell::RefCell::new(base);
            let end = (base as *mut u8).offset(size as isize) as *mut BufferBlock;    
            Self { layout, base, last: std::cell::RefCell::new(std::ptr::null_mut()), tail, end, block_count: 0 }       
        }        
    }

    fn iter_blocks(&self) -> BufferBlockIterator {
        if *self.tail.borrow() == self.base {
            return BufferBlockIterator(std::ptr::null_mut());
        } else {
            return BufferBlockIterator(self.base);
        }
    }

    pub fn iter(&self) -> BufferCellIterator {
        BufferCellIterator(self.iter_blocks(), Default::default())
    }

    /// Find a candidate block (unshared and which lru is low) to reallocate
    fn find_candidate_block(&self, size: usize) -> Option<*mut BufferBlock>
    {
        self.iter_blocks()
        .filter(|ptr| {
            unsafe {
                let block_ref = ptr.as_ref().unwrap();
                block_ref.is_unshared() 
                && block_ref.match_size(size) 
                && !block_ref.is_modified()
            }
        }).min_by_key(|ptr| {
            unsafe {
                let block_ref = ptr.as_ref().unwrap();
                block_ref.lru()
            }
        })
    }

    /// Find a free block
    fn find_free_block(&self, size: usize) -> Option<*mut BufferBlock>
    {
        self
        .iter_blocks()
        .find(|block_ptr| {
            unsafe {
                let block_ref = block_ptr.as_ref().unwrap();
                block_ref.is_free() && block_ref.match_size(size)
            }
        })
    }

    unsafe fn push_block(&self, size: usize) -> Result<*mut BufferBlock> 
    {
        let new_tail = (*self.tail.borrow() as *mut u8).offset(BufferBlock::size_of(size) as isize) as *mut BufferBlock;  
        
        if new_tail >= self.end {
            return Err(Error::NotEnoughSpace);
        }

        let new_block = *self.tail.borrow();
        *new_block = BufferBlock::new(size);      

        let last = *self.last.borrow();

        if !last.is_null() {
            (*last).next = new_block;    
        }

        *self.last.borrow_mut() = new_block;
        *self.tail.borrow_mut() = new_tail;

        Ok(new_block)
    }

    unsafe fn push_block_or_free_candidate(&self, size: usize) -> Result<*mut BufferBlock>
    {       
        match self.push_block(size) {
            Err(Error::NotEnoughSpace) => {
                
                if let Some(block) = self.find_candidate_block(size) 
                {
                    return Ok(block);
                }
                
                return Err(Error::NotEnoughSpace);
            },
            other => other
        }
    }

    pub fn alloc_raw(&self, size: usize) -> Result<*mut BufferBlock>
    {
        unsafe {
            let block = if let Some(block) = self.find_free_block(size) {
                block
            } else {
                self.push_block_or_free_candidate(size)?
            };
            return Ok(block)
        }
    }

    pub fn alloc<T>(&self, value: T) -> Result<BufferCell<T>> 
    {
        let mut block = self.alloc_uninit::<T>()?;
        *block = value;
        return Ok(block);
    }

    pub fn alloc_uninit<T>(&self)  -> Result<BufferCell<T>> {
        let block = self.alloc_raw(std::mem::size_of::<T>())?;
        Ok(BufferCell::new(block))      
    }

    pub fn alloc_array_uninit<'a, T>(&'a self, len: usize) -> Result<ArrayBufferCell<'a, T>> {
        let block = self.alloc_raw(std::mem::size_of::<T>().wrapping_mul(len))?; 
        Ok(ArrayBufferCell::new(block, len))
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            std::alloc::dealloc(self.base as *mut u8, self.layout);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ops::{DerefMut, Deref};
    use crate::fixtures;
    use super::{Buffer};

    #[test]
    fn test_buffer() -> super::Result<()> {
        let random = fixtures::random_data(16000);
        let buffer = Buffer::new_by_array::<u8>(16000, 300);
        let mut arr = buffer.alloc_array_uninit::<u8>(16000)?;

        assert_eq!(arr.deref().len(), 16000);
        arr.deref_mut().copy_from_slice(&random);
        
        assert_eq!(*arr.deref(), *random);
        
        Ok(())
    }
}


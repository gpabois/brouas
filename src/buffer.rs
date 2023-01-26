use std::{alloc::Layout, ops::{Deref, DerefMut}, borrow::BorrowMut};

#[derive(Debug)]
pub enum Error {
    NotEnoughSpace,
    MutablyBorrowed
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
        let cell = Self {
            block,
            _pht: Default::default()
        };

        cell.inc_rc();
        cell
    }

    pub fn leak(&self) -> *mut BufferBlock {
        self.block
    }

    pub fn leak_mut(&mut self) -> *mut BufferBlock {
        self.raise_upserted();
        self.block
    }

    pub fn inc_rc(&self) {
        unsafe {
            (*self.block).rc += 1;
        }
    }

    pub fn dec_rec(&self) {
        unsafe {
            (*self.block).rc -= 1;
        }
    }

    pub fn rc(&self) -> usize {
        unsafe {
            (*self.block).rc
        }
    }

    pub fn raise_upserted(&mut self) {
        unsafe {
            (*self.block).upserted = true;
        }       
    }
    /// Remove the modification flag
    pub fn drop_upserted(&mut self) {
        unsafe {
            (*self.block).upserted = false;
        }
    }

    pub fn is_upserted(&self) -> bool {
        unsafe {
            (*self.block).is_upserted()
        }
    }

    pub fn is_mut_borrowed(&self) -> bool {
        unsafe {
            (*self.block).is_mut_borrowed()
        }
    }

    pub fn raise_mut_borrow(&self) {
        unsafe {
            (*self.block).raise_mut_borrow();
        }
    }

    pub fn drop_mut_borrow(&self) {
        unsafe {
            (*self.block).drop_mut_borrow();
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

    pub fn try_into_array<T>(self) -> Option<BufArray<'buffer, T>> {
        unsafe {
            let size = std::mem::size_of::<T>();
            let block_size = (*self.block).size;

            let capacity = block_size.wrapping_div_euclid(size);
            let rem = block_size.wrapping_rem_euclid(size);

            if rem == 0 && capacity > 0 {
                return Some(BufArray::new(self, capacity))
            } else {
                return None
            }
        }
    }
}

impl<'buffer> Drop for RawBufferCell<'buffer> {
    fn drop(&mut self) {
        self.dec_rec()
    }
}

pub struct BufferCell<'buffer, T: ?Sized> {
    raw: RawBufferCell<'buffer>,
    _pht: std::marker::PhantomData<T>
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

/// A cell to an array stored in a buffer.
#[derive(Clone)]
pub struct BufArray<'buffer, T> {
    len: usize,
    raw: RawBufferCell<'buffer>,
    _pht: std::marker::PhantomData<T>
}

impl<'buffer, T> BufArray<'buffer, T> {
    fn new(raw: impl Into<RawBufferCell<'buffer>>, len: usize) -> Self {
        Self {
            len,
            raw: raw.into(),
            _pht: Default::default()
        }
    }

    /// Return an immutable reference to the array stored in the buffer, unless it is already mutable borrowed.
    pub fn try_borrow(&self) -> Result<RefBufArray<'buffer, T>> {
        if self.raw.is_mut_borrowed() {
            return Err(Error::MutablyBorrowed);
        } else {
            return Ok(
                RefBufArray::new(self.raw.clone(), self.len)
            )
        }
    }

    /// Return an immutable reference to the array stored in the buffer, panic if it is already mutable borrowed.
    pub fn borrow(&self) -> RefBufArray<'buffer, T> {
        self.try_borrow().unwrap()
    }

    /// Return a mutable reference to the array stored in the buffer, unless it is already mutable borrowed.
    pub fn try_borrow_mut(&self) -> Result<RefMutBufArray<'buffer, T>> {
        if self.raw.is_mut_borrowed() {
            return Err(Error::MutablyBorrowed);
        } else {
            return Ok(
                RefMutBufArray::new(self.raw.clone(), self.len)
            )
        }
    }

    /// Return a mutable reference to the array stored in the buffer, panic if it is already mutable borrowed.
    pub fn borrow_mut(&self) -> RefMutBufArray<'buffer, T> {
        self.try_borrow_mut().unwrap()
    }

    pub fn is_upserted(&self) -> bool {
        self.raw.is_upserted()
    }

    pub fn ack_upsertion(&mut self) {
        self.raw.drop_upserted()
    }
}

/// Immutable ref to an array stored in a buffer.
#[derive(Clone)]
pub struct RefBufArray<'buffer, T> {
    len: usize,
    raw: RawBufferCell<'buffer>,
    _pht: std::marker::PhantomData<T>
}

impl<'buffer, T> RefBufArray<'buffer, T> {
    fn new(raw: RawBufferCell<'buffer>, len: usize) -> Self {
        Self {
            len,
            raw,
            _pht: Default::default()
        }
    }
}

impl<'buffer, T> Deref for RefBufArray<'buffer, T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        unsafe {
            std::slice::from_raw_parts(BufferBlock::leak_value_unchecked::<T>(self.raw.leak()), self.len)
        }
    }
}

/// Mutable ref to an array stored in a buffer.
pub struct RefMutBufArray<'buffer, T> {
    len: usize,
    raw: RawBufferCell<'buffer>,
    _pht: std::marker::PhantomData<T>
}

impl<'buffer, T> RefMutBufArray<'buffer, T> {
    fn new(raw: RawBufferCell<'buffer>, len: usize) -> Self {
        let mut mut_ref = Self {
            len,
            raw,
            _pht: Default::default()
        };

        mut_ref.raw.raise_mut_borrow();
        mut_ref
    }

    pub fn degrade(self) -> RefBufArray<'buffer, T> {
        RefBufArray::new(self.raw, self.len)
    }
}

impl<'buffer, T> Drop for RefMutBufArray<'buffer, T> {
    fn drop(&mut self) {
        self.raw.drop_upserted();
    }
}

impl<'buffer, T> Deref for RefMutBufArray<'buffer, T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        unsafe {
            std::slice::from_raw_parts(BufferBlock::leak_value_unchecked::<T>(self.raw.leak()), self.len)
        }
    }
}

impl<'buffer, T> DerefMut for RefMutBufArray<'buffer, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            std::slice::from_raw_parts_mut(BufferBlock::leak_value_unchecked::<T>(self.raw.leak_mut()), self.len)
        }
    }
}
pub struct BufferBlock {
    pub size:           usize,
    pub rc:             usize,
    pub lru:            usize,
    pub free:           bool,
    pub upserted:       bool,
    pub mut_borrowed:   bool,
    pub next:           *mut BufferBlock
}

impl BufferBlock {
    pub fn new(size: usize) -> Self {
        Self {
            size,
            free: false,
            rc: 0,
            lru: 0,
            upserted: false,
            mut_borrowed: false,
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

    pub fn is_upserted(&self) -> bool {
        return self.upserted
    }

    pub fn is_mut_borrowed(&self) -> bool {
        self.mut_borrowed
    }

    pub fn drop_mut_borrow(&mut self) {
        self.mut_borrowed = false;
    }

    pub fn raise_mut_borrow(&mut self) {
        self.mut_borrowed = true;
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

pub struct BufCellIterator<'buffer>(BufferBlockIterator, std::marker::PhantomData<&'buffer ()>);

impl<'buffer> Iterator for BufCellIterator<'buffer> {
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

    pub fn iter(&self) -> BufCellIterator {
        BufCellIterator(self.iter_blocks(), Default::default())
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
                && !block_ref.is_upserted()
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

    pub fn alloc_array_uninit<'a, T>(&'a self, len: usize) -> Result<BufArray<'a, T>> {
        let block = self.alloc_raw(std::mem::size_of::<T>().wrapping_mul(len))?; 
        Ok(BufArray::new(block, len))
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
    use crate::fixtures;
    use super::{Buffer};

    #[test]
    fn test_buffer() -> super::Result<()> {
        let random = fixtures::random_data(16000);
        let buffer = Buffer::new_by_array::<u8>(16000, 300);
        let mut arr = buffer.alloc_array_uninit::<u8>(16000)?;

        assert_eq!(arr.borrow().len(), 16000);
        arr.borrow_mut().copy_from_slice(&random);
        
        assert_eq!(*arr.borrow(), *random);
        
        Ok(())
    }
}


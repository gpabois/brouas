use std::{alloc::Layout, ops::{Deref, DerefMut}, marker::PhantomData};

#[derive(Debug)]
pub enum Error {
    NotEnoughSpace
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct BufferCell<T> {
    block: *mut BufferBlock,
    _phantom: std::marker::PhantomData<T>
}

impl<T> Deref for BufferCell<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe {
            BufferBlock::leak_value_unchecked::<T>(self.block).as_ref().unwrap()
        }
    }
}

impl<T> DerefMut for BufferCell<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            (*self.block).modified = true;
            BufferBlock::leak_value_unchecked::<T>(self.block).as_mut().unwrap()
        }
    }
}

impl<T> BufferCell<T> 
{
    pub fn new(block: *mut BufferBlock) -> Self {
        unsafe {
            (*block).rc += 1;
        }

        Self {
            block,
            _phantom: Default::default()
        }
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
}

impl<T> Drop for BufferCell<T> {
    fn drop(&mut self) {
        unsafe {
            (*self.block).rc -= 1;
        }
    }
}

impl<T> Clone for BufferCell<T> {
    fn clone(&self) -> Self {
        Self::new(self.block)
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

impl Buffer {
    /// Create a buffer intented to be used with equal_sized memory blocks.
    pub fn new_equal_blocks<T>(capacity: usize) -> Self {
        let align   = (std::mem::size_of::<T>() + std::mem::size_of::<BufferBlock>()).next_power_of_two();
        let size    = capacity.wrapping_mul(align);
        let layout = Layout::from_size_align(size, align).unwrap();
        
        unsafe {
            let base = std::alloc::alloc_zeroed(layout) as *mut BufferBlock;
            let tail = std::cell::RefCell::new(base);
            let end = base.offset(size as isize);    
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

    /// Find a candidate block (unshared and which lru is low) to reallocate
    fn find_candidate_block(&self, size: usize) -> Option<*mut BufferBlock>
    {
        let mut blocks: Vec<_> = self.iter_blocks()
        .filter(|ptr| {
            unsafe {
                let block_ref = ptr.as_ref().unwrap();
                block_ref.is_unshared() 
                && block_ref.match_size(size) 
                && !block_ref.is_modified()
            }
        }).collect();

        blocks.sort_by_key(|ptr| {
            unsafe {
                let block_ref = ptr.as_ref().unwrap();
                block_ref.lru()
            }
        });

        blocks.first().copied()
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
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            std::alloc::dealloc(self.base as *mut u8, self.layout);
        }
    }
}

pub struct BufferPool<T>(Buffer, PhantomData<T>);

impl<T> BufferPool<T>
where T: 'static 
{
    pub fn new(capacity: usize) -> Self {
        Self(Buffer::new_equal_blocks::<T>(capacity), Default::default())
    }

    pub fn alloc(&self, value: T) -> Result<BufferCell<T>> {
        self.0.alloc(value)
    }

    pub fn alloc_uninit(&self) -> Result<BufferCell<T>> {
        self.0.alloc_uninit::<T>()
    }

    pub fn iter(&self) -> BufferPoolIterator<T> {
        BufferPoolIterator(self.0.iter_blocks(), Default::default())
    }
}

pub struct BufferPoolIterator<T>(BufferBlockIterator, std::marker::PhantomData<T>);

impl<T> Iterator for BufferPoolIterator<T> {
    type Item = BufferCell<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(block) = self.0.next() {
            return Some(BufferCell::new(block))
        } else {
            return None
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ops::{DerefMut, Deref};
    use crate::fixtures;
    use super::{Buffer, BufferPool};

    #[test]
    fn test_buffer() -> super::Result<()> {
        let random = fixtures::random_data(16000);
        let buffer = Buffer::new_equal_blocks::<[u8;16000]>(200);
        let mut arr = buffer.alloc_uninit::<[u8;16000]>()?;

        arr.deref_mut().copy_from_slice(&random);
        
        assert_eq!(*arr.deref(), *random);
        
        Ok(())
    }

    #[test]
    fn test_buffer_pool() -> super::Result<()> {
        let pool = BufferPool::<[u8; 16_000]>::new(100);

        for _ in 0..100 {
            let mut block = pool.alloc_uninit()?;
            fixtures::randomise(block.deref_mut());
        }

        Ok(())
    }
}


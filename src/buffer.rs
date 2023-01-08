use std::alloc::Layout;

pub struct MemBlock {
    pub size: usize,
    pub free: bool,
    pub next: *mut MemBlock
}

impl MemBlock {
    pub unsafe fn get_block(block: *mut Self) -> *mut u8 {
        block.offset(std::mem::size_of::<Self>() as isize) as *mut u8
    }
}

pub struct Buffer {
    pub layout: Layout,
    pub base: *mut u8,
    pub block_count: usize
}

pub struct MemBlockIterator(*mut MemBlock);

impl Iterator for MemBlockIterator {
    type Item = *mut MemBlock;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0.is_null() {
            return None;
        } else {
            let blck = self.0;
            unsafe {
                self.0 = (*self.0).next as *mut MemBlock;
            }
            return Some(blck);
        }
    }
}

impl Buffer {
    /// Create a buffer intented to be used with equal_sized memory blocks.
    pub fn new_equal_blocks<T>(capacity: usize) -> Self {
        let align = std::mem::size_of::<T>() + std::mem::size_of::<MemBlock>();
        let size = capacity.wrapping_mul(align);
        let layout = Layout::from_size_align(size, align).unwrap();
        
        unsafe {
            let base = std::alloc::alloc_zeroed(layout);    
            Self { layout, base, block_count: 0 }       
        }
    }

    pub fn iter(&self) -> MemBlockIterator {
        if self.block_count == 0 {
            return MemBlockIterator(std::ptr::null_mut() as *mut MemBlock);
        } else {
            return MemBlockIterator(self.base as *mut MemBlock);
        }
    }

    pub fn alloc<T>(&self, value: T) {
        let size = std::mem::size_of::<T>();
        self.iter().find(|mem| (**mem).free && (**mem).size == size);

    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            std::alloc::dealloc(self.base, self.layout);
        }
    }
}







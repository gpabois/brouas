use crate::io::Data;

use self::{buffer::ObjectsBuffer, io::traits::{ObjectRead, ObjectWrite, ObjectsWrite, ObjectsRead}, counter::ObjectCounter, meta::ObjectMeta};

pub mod io;
pub mod buffer;
pub mod counter;
pub mod meta;
pub mod error;
pub mod result;

pub type ObjectId = u64;
pub type ObjectType = u16;
pub type ObjectSize = usize;

/// Reserved object types
pub const BPTREE_BRANCH: u16 = 0x0010;
pub const BPTREE_LEAF: u16   = 0x0011;

pub struct Objects<S> {
    counter: ObjectCounter,
    buffer: ObjectsBuffer,
    stream: S
}

impl<S> Objects<S>{
    pub fn new(stream: S) -> Self {
        Self {
            counter: Default::default(),
            buffer: ObjectsBuffer::new(),
            stream
        }
    }
}

impl<S> ObjectWrite for Objects<S> 
{
    fn write_object<O>(&mut self, content: &O, meta: meta::ObjectMeta) -> result::Result<()> 
    where O: crate::io::traits::OutStream<Output=O> {
        self.buffer.write_object(content, meta)
    }
}

impl<S> ObjectRead for Objects<S> 
where S: ObjectsRead
{
    fn read_object<O>(&mut self, object: &mut O, requirement: meta::ObjectRequirement) -> result::Result<ObjectMeta>
        where O: crate::io::traits::InStream<Input=O> {
        if self.buffer.contains(&requirement.get_id()) == false {
            let mut data = Data::new();
            let meta = self.stream.read_object(&mut data, requirement)?;
            self.buffer.cache_object(&data, meta)?;
        }

        self.buffer.read_object(object, requirement)
    }
}

impl<S> Objects<S> 
where S: ObjectsRead + ObjectsWrite
{
    /// Clear the current buffer
    pub fn clear_buffer(&mut self) {
        self.buffer.clear();
    }

    pub fn flush_objects(&mut self) -> self::result::Result<()> {
        self.stream.write_objects_header(&self.counter)?;
        self.buffer.flush_objects(&mut self.stream)?;
        self.stream.flush_objects();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{fixtures, io::Data, object::{io::ObjectsFile, Objects, meta::ObjectRequirement}};
    use super::{io::traits::{ObjectWrite, ObjectRead}, meta::ObjectMeta};
    
    #[test]
    fn test_objects() -> super::result::Result<()> {
        let mut objects = Objects::new(ObjectsFile::open("working"));

        let meta = ObjectMeta::new(100, 0x33);
        let data = fixtures::random_data(100);
        let mut stored = Data::with_size(100usize);

        objects.write_object(&data, meta)?;
        objects.flush_objects()?;

        // Clear the buffer, so we fetch object from the stream.
        objects.clear_buffer();
        objects.read_object(&mut stored, ObjectRequirement::new(100, 0x33))?;

        assert_eq!(stored, data);

        Ok(())
    }
}

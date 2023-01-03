use self::{buffer::ObjectsBuffer, io::traits::{ObjectRead, ObjectWrite}, counter::SharedObjectCounter};

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

pub struct Objects<S> 
where S: ObjectRead
{
    counter: SharedObjectCounter,
    buffer: ObjectsBuffer,
    ro_stream: S
}

impl<S> Objects<S> 
where S: ObjectRead
{
    pub fn flush<OW: ObjectWrite>(&mut self, write: &mut OW) -> self::result::Result<()> {
        self.buffer.flush_objects(write)
    }
}

#[cfg(test)]
mod tests {
    use crate::{fixtures, io::Data};
    use super::{io::{InMemoryObjects, traits::{ObjectWrite, ObjectRead}}, meta::ObjectMeta};
    
    #[test]
    fn test_objects() -> super::result::Result<()> {
        let mut objects = InMemoryObjects::new();

        let meta = ObjectMeta::new(100, 0x33);
        let data = fixtures::random_data(100);
        let mut stored = Data::with_size(100usize);

        objects.write_object(&data, meta)?;
        objects.read_object(&mut stored, meta)?;

        assert_eq!(stored, data);

        Ok(())
    }
}

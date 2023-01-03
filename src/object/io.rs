use std::io::Cursor;
use std::ops::Deref;

use self::traits::{ObjectRead, ObjectWrite};
use crate::io::Data;
use crate::io::traits::InStream;
use super::buffer::ObjectsBuffer;
use super::meta::ObjectMeta;
use super::result::Result;
use super::error::Error;

pub mod traits;

pub type InMemoryObjects = ObjectsBuffer;

pub struct SledObjectsStream (sled::Tree);

impl ObjectRead for SledObjectsStream {
    fn read_object<O>(&mut self, object: &mut O, meta: ObjectMeta) -> Result<()>
    where O: InStream<Input=O> {
        let key = meta.get_id().to_le_bytes();

        let raw = self.0.get(&key)
        .unwrap()
        .ok_or(Error::ObjectNotFound)?;

        let mut data = Cursor::new(raw.as_ref());

        O::read_from_stream(object, &mut data)?;

        Ok(())
    }
}

impl ObjectWrite for SledObjectsStream {
    fn write_object<O>(&mut self, content: &O, meta: ObjectMeta) -> Result<()> 
    where O: crate::io::traits::OutStream<Output=O> {
        let key = meta.get_id().to_le_bytes();
        let mut data = Data::new();
        O::write_all_to_stream(content, &mut data)?;
        self.0.insert(key, data.deref()).unwrap();
        Ok(())
    }
}
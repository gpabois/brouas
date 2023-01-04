use std::{collections::BTreeMap, ops::Deref};
use crate::{io::{Data, traits::{OutStream, InStream}}, utils::Watcher};
use super::{error::Error, result::Result, meta::{ObjectMeta, ObjectRequirement}, io::traits::{ObjectRead, ObjectWrite}, ObjectId};

pub struct ObjectBuffer {
    meta: ObjectMeta,
    data: Data
}

impl ObjectBuffer {
    pub fn new(meta: ObjectMeta, data: Data) -> Self {
        Self {meta, data}
    }
}

impl InStream for ObjectBuffer {
    type Input = Self;

    fn read_from_stream<R: std::io::Read + ?Sized>(input: &mut Self::Input, read: &mut R) -> std::io::Result<()> {
        read.read_exact(&mut input.data)
    }
}

impl OutStream for ObjectBuffer {
    type Output = Self;

    fn write_to_stream<W: std::io::Write + ?Sized>(output: &Self::Output, writer: &mut W) -> std::io::Result<usize> {
        writer.write(&output.data)
    }

    fn write_all_to_stream<W: std::io::Write + ?Sized>(output: &Self::Output, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&output.data)
    }
}

pub struct ObjectsBuffer {
    table: BTreeMap<ObjectId, Watcher<ObjectBuffer>>
}

impl ObjectsBuffer {
    pub fn new() -> Self {
        Self { table: Default::default() }
    }
    /// Flush upserted objects
    pub fn flush_objects<OW: ObjectWrite>(&mut self, write: &mut OW) -> Result<()> {
        self.table
        .iter_mut()
        .filter(|kv| kv.1.is_modified())
        .map(|kv| {
           kv.1.done();
           write.write_object(&kv.1.data, kv.1.meta.clone())
        }).collect::<Result<Vec<_>>>()?;

        Ok(())
    }

    pub fn contains(&self, id: &ObjectId) -> bool {
        self.table.contains_key(id)
    }

    pub fn clear(&mut self) {
        self.table.clear()
    }

    pub fn cache_object<O>(&mut self, content: &O, meta: ObjectMeta) -> Result<()> 
    where O: crate::io::traits::OutStream<Output=O> {
        let oid = meta.get_id();
        let mut data = Data::new();
        // Write the rest of the object
        O::write_all_to_stream(content, &mut data)?;
        // Insert in the buffer
        self.table.insert(oid, Watcher::wrap(ObjectBuffer { meta: meta, data: data }));
        Ok(())
    }
}

impl ObjectWrite for ObjectsBuffer {
    fn write_object<O>(&mut self, content: &O, meta: ObjectMeta) -> Result<()> 
    where O: crate::io::traits::OutStream<Output=O> {
        let oid = meta.get_id();
        let mut data = Data::new();
        // Write the rest of the object
        O::write_all_to_stream(content, &mut data)?;
        // Insert in the buffer
        self.table.insert(oid, Watcher::new(ObjectBuffer { meta: meta, data: data }));
        Ok(())
    }
}

impl ObjectRead for ObjectsBuffer {
    fn read_object<O>(&mut self, object: &mut O, meta: ObjectRequirement) -> Result<()> 
    where O: crate::io::traits::InStream<Input=O>
    {
        let oid = meta.get_id();
        
        let obuffer = self.table.get(&oid)
        .ok_or(Error::ObjectNotFound)?
        .deref();
        
        // Get a read cursor 
        let mut read = obuffer.data.get_cursor_read();

        // Assert object type
        meta.assert_type(&obuffer.meta.get_type())?;

        // Read the rest
        O::read_from_stream(object, &mut read)?;

        Ok(())
    }
}
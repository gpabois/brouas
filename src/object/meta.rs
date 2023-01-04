use crate::io::{DataStream, Data};
use crate::io::traits::{OutStream, InStream};

use super::error::Error;
use super::{ObjectId, ObjectType};
use super::result::Result;

pub struct ObjectRequirement {
    id: ObjectId,
    otype: ObjectType
}

impl ObjectRequirement {
    pub fn new(id: ObjectId, otype: ObjectType) -> Self {
        Self{id, otype}
    }

    pub fn get_id(&self) -> ObjectId {
        self.id
    }

    pub fn assert(&self, meta: &ObjectMeta) -> Result<()> {
        if self.otype != meta.otype {
            return Err(Error::InvalidObjectType { expected: self.otype, got: meta.otype })
        }
        Ok(())
    }
}

#[derive(Clone, Default)]
pub struct ObjectMeta {
    id: ObjectId,
    otype: ObjectType,
    parent: Option<ObjectId>,
    children: Vec<ObjectId>
}

impl InStream for ObjectMeta {
    type Input = Self;

    fn read_from_stream<R: std::io::Read + ?Sized>(input: &mut Self::Input, read: &mut R) -> std::io::Result<()> {
        input.id = DataStream::<ObjectId>::read(read)?;
        input.otype = DataStream::<ObjectType>::read(read)?;
        input.parent = match DataStream::<ObjectId>::read(read)? {
            0 => None,
            other => Some(other)
        };
        
        input.children = vec![0; DataStream::<u64>::read(read)? as usize];

        for coid in input.children.iter_mut() {
            *coid = DataStream::<ObjectId>::read(read)?;
        }

        Ok(())
    }
}

impl OutStream for ObjectMeta {
    type Output = Self;

    fn write_to_stream<W: std::io::Write + ?Sized>(output: &Self::Output, writer: &mut W) -> std::io::Result<usize> {
        let written = DataStream::<ObjectId>::write(writer, output.id)? +
        DataStream::<ObjectType>::write(writer, output.otype)? +
        DataStream::<ObjectId>::write(writer, output.parent.unwrap_or(0))? +
        DataStream::<u64>::write(writer, output.children.len() as u64)?;

        for coid in output.children.iter() {
            written += DataStream::<ObjectId>::write(writer, *coid)?;
        }

        Ok(written)
    }

    fn write_all_to_stream<W: std::io::Write + ?Sized>(output: &Self::Output, writer: &mut W) -> std::io::Result<()> {
        DataStream::<ObjectId>::write_all(writer, output.id)?;
        DataStream::<ObjectType>::write_all(writer, output.otype)?;
        DataStream::<ObjectId>::write_all(writer, output.parent.unwrap_or(0))?;
        DataStream::<u64>::write_all(writer, output.children.len() as u64)?;

        for coid in output.children.iter() {
            DataStream::<ObjectId>::write(writer, *coid)?;
        }

        Ok(())
    }
}

impl ObjectMeta {
    pub fn new(id: ObjectId, otype: ObjectType) -> Self {
        Self{
            id, 
            otype,
            parent: None,
            children: vec![]
        }
    }

    pub fn get_type(&self) -> ObjectType {
        self.otype
    }

    pub fn get_id(&self) -> ObjectId {
        self.id
    }
}

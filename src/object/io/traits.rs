use std::io::Write;

use crate::io::traits::{OutStream, InStream};
use crate::object::meta::{ObjectMeta, ObjectRequirement};
use crate::object::result::Result;


/// Write the objects, as well as the metadata (counter, ...)
pub trait ObjectsWrite: ObjectWrite {
    fn write_objects_header<H: OutStream<Output=H>>(&mut self, header: &H) -> Result<()>;
    fn flush_objects(&mut self);
}

pub trait ObjectsRead: ObjectRead {
    fn read_objects_header<H: InStream<Input=H>>(&mut self, header: &mut H) -> Result<()>;
}

/// Defines an object write stream.
pub trait ObjectWrite {
    fn write_object<O>(&mut self, content: &O, meta: ObjectMeta) -> Result<()> 
    where O: OutStream<Output=O>;
}

/// Defines an object read stream.
pub trait ObjectRead {
    fn read_object<O>(&mut self, object: &mut O, requirement: ObjectRequirement) -> Result<ObjectMeta>
    where O: InStream<Input=O>;
}
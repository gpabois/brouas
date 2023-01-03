use crate::io::traits::{OutStream, InStream};
use crate::object::meta::ObjectMeta;
use crate::object::result::Result;

/// Defines an object write stream.
pub trait ObjectWrite {
    fn write_object<O>(&mut self, content: &O, meta: ObjectMeta) -> Result<()> 
    where O: OutStream<Output=O>;
}

/// Defines an object read stream.
pub trait ObjectRead {
    fn read_object<O>(&mut self, object: &mut O, meta: ObjectMeta) -> Result<()>
    where O: InStream<Input=O>;
}
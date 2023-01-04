use std::io::Read;

use crate::io::Data;
use crate::io::lsm::Key;
use crate::io::traits::{InStream, OutStream, WriteStream, ReadStream};
use self::traits::{ObjectWrite, ObjectRead};

use super::buffer::ObjectsBuffer;
use super::meta::{ObjectMeta, ObjectRequirement};
use super::result::Result;
use super::error::Error;

pub mod traits;

pub type InMemoryObjects = ObjectsBuffer;

enum ObjectWriteHint {
    Header,
    Object(ObjectMeta)
}

enum ObjectReadHint {
    Header,
    Object(ObjectRequirement)
}

pub struct ObjectStream<S>(S);

impl<S> ReadStream for ObjectStream<S> where S: ReadStream<Hints=Key> {
    type Hints = ObjectReadHint;

    fn read<In: InStream<Input=In>>(&mut self, input: &mut In, hints: &Self::Hints) -> std::io::Result<()> {
        match hints {
            ObjectReadHint::Header => {
                self.0.read(input, &Key::from("objects"))
            },
            ObjectReadHint::Object(requirement) => {
                let mut data = Data::new();
                self.0.read(&mut data, &Key::from(requirement.get_id()))?;
                ObjectMeta::read_from_stream(input, read)
                Ok(())  
            },
        }
    }
}

impl<S> ObjectWrite for ObjectStream<S> where S: WriteStream<Hints=Key> {
    fn write_object<Out>(&mut self, out: &Out, meta: ObjectMeta) -> Result<()> 
    where Out: OutStream<Output=Out> {
        self.write_all(out, &ObjectWriteHint::Object(meta))?;
        Ok(())
    }
}

impl<S> WriteStream for ObjectStream<S> where S: WriteStream<Hints=Key> {
    type Hints = ObjectWriteHint;

    fn write<Out>(&mut self, out: &Out, hints: &Self::Hints) -> std::io::Result<usize> 
    where Out: OutStream<Output=Out>
    {
        match hints {
            ObjectWriteHint::Header => {
                self.0.write(out, &Key::from("objects"))
            },
            ObjectWriteHint::Object(meta) => {
                let mut data = Data::new();
                ObjectMeta::write_all_to_stream(meta, &mut data)?;
                Out::write_all_to_stream(out, &mut data)?;
                self.0.write(&data, &Key::from(meta.get_id()))
            },
        }
    }

    fn write_all<Out: OutStream<Output=Out>>(&mut self, out: &Out, hints: &Self::Hints) -> std::io::Result<()> {
        match hints {
            ObjectWriteHint::Header => {
                self.0.write_all(out, &Key::from("objects"))
            },
            ObjectWriteHint::Object(meta) => {
                let mut data = Data::new();
                ObjectMeta::write_all_to_stream(meta, &mut data)?;
                Out::write_all_to_stream(out, &mut data)?;
                self.0.write_all(&data, &Key::from(meta.get_id()))
            },
        }
    }

    fn flush(&mut self) -> std::io::Result<usize> {
        self.0.flush()
    }
}
use std::{cell::RefCell, rc::Rc, ops::Deref};

use crate::io::{traits::{OutStream, InStream}, DataStream};

use super::ObjectId;

#[derive(Clone, Default)]
pub struct ObjectCounter(Rc<RefCell<ObjectId>>);

impl ObjectCounter {
    pub fn new_id(&self) -> ObjectId {
        *self.0.deref().borrow_mut() += 1;
        *self.0.borrow()
    }
}

impl InStream for ObjectCounter {
    type Input = Self;

    fn read_from_stream<R: std::io::Read + ?Sized>(input: &mut Self::Input, read: &mut R) -> std::io::Result<()> {
        *input.0.borrow_mut() = DataStream::<ObjectId>::read(read)?;
        Ok(())
    }
}

impl OutStream for ObjectCounter {
    type Output = Self;

    fn write_to_stream<W: std::io::Write + ?Sized>(output: &Self::Output, writer: &mut W) -> std::io::Result<usize> {
        DataStream::<ObjectId>::write(writer, *output.0.borrow())
    }

    fn write_all_to_stream<W: std::io::Write + ?Sized>(output: &Self::Output, writer: &mut W) -> std::io::Result<()> {
        DataStream::<ObjectId>::write_all(writer, *output.0.borrow())
    }
}

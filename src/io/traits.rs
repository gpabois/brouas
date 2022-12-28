use std::io::{Write, BufRead};


/// Trait to write object into an output stream.
pub trait OutStream 
{
    fn write_to_stream<W: Write>(&self, writer: &mut W) -> std::io::Result<usize>;
    fn write_all_to_stream<W: Write>(&self, writer: &mut W) -> std::io::Result<()>; 
}

/// Trait to read the input stream and update the instance.
pub trait InStream 
{
    fn read_from_stream<R: BufRead>(&mut self, read: &mut R) -> std::io::Result<()>;
}


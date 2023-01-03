use std::io::{Write, Read};

/// Trait to write data into an output stream.
pub trait OutStream 
{
    type Output;
    
    fn write_to_stream<W: Write + ?Sized>(output: &Self::Output, writer: &mut W) -> std::io::Result<usize>;
    fn write_all_to_stream<W: Write + ?Sized>(output: &Self::Output, writer: &mut W) -> std::io::Result<()>; 
}

/// Trait to read the input stream and update the instance.
pub trait InStream 
{
    type Input;

    fn read_from_stream<R: Read + ?Sized>(input: &mut Self::Input, read: &mut R) -> std::io::Result<()>;
}

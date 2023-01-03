use std::{io::{Write, Read, BufWriter, Cursor, Seek}, marker::PhantomData, ops::{DerefMut, Deref}, cmp::min};

use self::traits::{OutStream, InStream};

pub mod traits;

pub struct DataStream<T>(PhantomData<T>);

impl DataStream<u64> {
    pub fn read<R: Read + ?Sized>(read: &mut R) -> std::io::Result<u64> {
        read_u64(read)
    }

    pub fn write_all<W: Write + ?Sized>(writer: &mut W, value: u64) -> std::io::Result<()> {
        writer.write_all(&value.to_ne_bytes())
    }

    pub fn write<W: Write + ?Sized>(writer: &mut W, value: u64) -> std::io::Result<usize> {
        writer.write(&value.to_ne_bytes())
    }
}

impl DataStream<u32> {
    pub fn read<R: Read>(read: &mut R) -> std::io::Result<u32> {
        read_u32(read)
    }

    pub fn write<W: Write>(writer: &mut W, value: u32) -> std::io::Result<usize> {
        writer.write(&value.to_ne_bytes())
    }

    pub fn write_all<W: Write>(writer: &mut W, value: u32) -> std::io::Result<()> {
        writer.write_all(&value.to_ne_bytes())
    }
}

impl DataStream<u16> {
    pub fn read<R: Read>(read: &mut R) -> std::io::Result<u16> {
        read_u16(read)
    }

    pub fn write<W: Write>(writer: &mut W, value: u16) -> std::io::Result<usize> {
        writer.write(&value.to_ne_bytes())
    }

    pub fn write_all<W: Write>(writer: &mut W, value: u16) -> std::io::Result<()> {
        writer.write_all(&value.to_ne_bytes())
    }
}

impl DataStream<u8> {
    pub fn read<R: Read>(read: &mut R) -> std::io::Result<u8> {
        read_u8(read)
    }
    pub fn write<W: Write>(writer: &mut W, value: u8) -> std::io::Result<usize> {
        writer.write(&value.to_ne_bytes())
    }

    pub fn write_all<W: Write>(writer: &mut W, value: u8) -> std::io::Result<()> {
        writer.write_all(&value.to_ne_bytes())
    }
}

fn read_u64<R: Read + ?Sized>(read: &mut R) -> std::io::Result<u64> {
    let mut value: [u8; 8] = [0; 8];
    read.read_exact(&mut value)?;
    Ok(u64::from_le_bytes(value))
}

fn read_u32<R: Read>(read: &mut R) -> std::io::Result<u32> {
    let mut value: [u8; 4] = [0; 4];
    read.read_exact(&mut value)?;
    Ok(u32::from_le_bytes(value))
}

fn read_u16<R: Read>(read: &mut R) -> std::io::Result<u16> {
    let mut value: [u8; 2] = [0; 2];
    read.read_exact(&mut value)?;
    Ok(u16::from_le_bytes(value))
}

fn read_u8<R: Read>(read: &mut R) -> std::io::Result<u8> {
    let mut value: [u8; 1] = [0; 1];
    read.read_exact(&mut value)?;
    Ok(u8::from_le_bytes(value))
}

pub struct InMemory(Cursor<Vec<u8>>);

impl InMemory {
    pub fn new() -> Self {
        Self(Default::default())
    }
}

impl Deref for InMemory 
{
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.0.get_ref()
    }
}

impl DerefMut for InMemory {
    fn deref_mut(&mut self) -> &mut Self::Target 
    {
        self.0.get_mut()
    }
}


impl std::io::Write for InMemory 
{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> 
    {
        self.0.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> 
    {
        self.0.flush()
    }
}

impl std::io::Read for InMemory 
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> 
    {
        self.0.read(buf)
    }
}

impl std::io::Seek for InMemory {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        self.0.seek(pos)
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Data(Vec<u8>);

impl From<Vec<u8>> for Data 
{
    fn from(value: Vec<u8>) -> Self 
    {
        Self(value)
    }
}

impl Deref for Data 
{
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Data {
    fn deref_mut(&mut self) -> &mut Self::Target 
    {
        &mut self.0
    }
}

impl Write for Data 
{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> 
    {
        self.0.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> 
    {
        Ok(())
    }
}

impl std::io::Read for Data 
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> 
    {
        let consumed: Vec<_> = self.0.drain(0..buf.len()).collect();
        buf[..consumed.len()].copy_from_slice(&consumed);
        Ok(consumed.len())
    }
}

impl Data 
{
    pub fn new() -> Self {
        Self(vec![])
    }

    pub fn with_size(size: impl Into<usize>) -> Self {
        Self(vec![0; size.into()])
    }
    
    /// Pop at more nb_bytes from the buffer and returns it in dedicated buffer
    pub fn pop_front(&mut self, nb_bytes: impl Into<usize>) -> Data 
    {
        let mut nb_bytes: usize  = nb_bytes.into();
        nb_bytes = min::<usize>(self.len(), nb_bytes);
        Data(self.0.drain(0..nb_bytes).collect())
    }

    pub fn increase_size_if_necessary(&mut self, size: usize) {
        if size > self.len() 
        {
            self.0.resize(size, 0);
        }
    }

    pub fn extend_from_slice(&mut self, data: &[u8]) 
    {
        self.0.extend_from_slice(data);
    }

    pub fn len(&self) -> usize 
    {
        self.0.len()
    }

    pub fn get_cursor_write(&mut self) -> BufWriter<Cursor<&mut [u8]>> 
    {
        BufWriter::new(Cursor::new(&mut self.0))       
    }

    pub fn get_cursor_read(&self) -> DataReadBuffer<'_>
    {
        Cursor::new(&self.0)
    }

    pub fn is_empty(&self) -> bool 
    {
        self.0.is_empty()
    }
}

pub type DataReadBuffer<'a> = Cursor<&'a[u8]>;

pub struct DataRef<'a>(&'a [u8]);

impl<'a> DataRef<'a> {
    pub fn new(r: &'a [u8]) -> Self {
        Self(r)
    }
}

impl<'a> OutStream for DataRef<'a>
{
    type Output = Self;

    fn write_to_stream<W: std::io::Write + ?Sized>(output: &Self::Output, writer: &mut W) -> std::io::Result<usize> {
        writer.write(&output.0)
    }

    fn write_all_to_stream<W: Write + ?Sized>(output: &Self::Output, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&output.0)       
    }
}


impl OutStream for Data {
    type Output = Self;

    fn write_to_stream<W: std::io::Write + ?Sized>(output: &Self, writer: &mut W) -> std::io::Result<usize> {
        writer.write(output)
    }

    fn write_all_to_stream<W: Write + ?Sized>(output: &Self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(output)       
    }
}

impl InStream for Data {
    type Input = Self;
    
    fn read_from_stream<R: std::io::Read + ?Sized>(input: &mut Self, read: &mut R) -> std::io::Result<()> {
        read.read_exact(input)
    }
}

pub fn is_empty<S: Seek>(stream: &mut S) -> std::io::Result<bool> {
    let cursor = stream.stream_position()?;
    let end = stream.seek(std::io::SeekFrom::End(0))?;

    stream.seek(std::io::SeekFrom::Start(cursor))?;
    
    Ok(cursor == end)
}
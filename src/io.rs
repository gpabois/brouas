use std::{io::{Write, Read, BufWriter, Cursor, BufReader}, marker::PhantomData, ops::{DerefMut, Deref}, cmp::min};

use self::traits::{OutStream, InStream};

pub mod traits;

pub struct DataStream<T>(PhantomData<T>);

impl DataStream<u64> {
    pub fn read<R: Read>(read: &mut R) -> std::io::Result<u64> {
        read_u64(read)
    }

    pub fn write_all<W: Write>(writer: &mut W, value: u64) -> std::io::Result<()> {
        writer.write_all(&value.to_ne_bytes())
    }

    pub fn write<W: Write>(writer: &mut W, value: u64) -> std::io::Result<usize> {
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

fn read_u64<R: Read>(read: &mut R) -> std::io::Result<u64> {
    let mut value: [u8; 8] = [0; 8];
    read.read_exact(&mut value)?;
    Ok(u64::from_ne_bytes(value))
}

fn read_u32<R: Read>(read: &mut R) -> std::io::Result<u32> {
    let mut value: [u8; 4] = [0; 4];
    read.read_exact(&mut value)?;
    Ok(u32::from_ne_bytes(value))
}

fn read_u16<R: Read>(read: &mut R) -> std::io::Result<u16> {
    let mut value: [u8; 2] = [0; 2];
    read.read_exact(&mut value)?;
    Ok(u16::from_ne_bytes(value))
}

fn read_u8<R: Read>(read: &mut R) -> std::io::Result<u8> {
    let mut value: [u8; 1] = [0; 1];
    read.read_exact(&mut value)?;
    Ok(u8::from_ne_bytes(value))
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

impl std::io::BufRead for InMemory 
{
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> 
    {
        self.0.fill_buf()
    }

    fn consume(&mut self, amt: usize) 
    {
        self.0.consume(amt)
    }
}

impl std::io::Seek for InMemory {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        self.0.seek(pos)
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct DataBuffer(Vec<u8>);

impl From<Vec<u8>> for DataBuffer 
{
    fn from(value: Vec<u8>) -> Self 
    {
        Self(value)
    }
}

impl Deref for DataBuffer 
{
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for DataBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target 
    {
        &mut self.0
    }
}

impl Write for DataBuffer 
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

impl std::io::Read for DataBuffer 
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> 
    {
        let consumed: Vec<_> = self.0.drain(0..buf.len()).collect();
        buf[..consumed.len()].copy_from_slice(&consumed);
        Ok(consumed.len())
    }
}

impl std::io::BufRead for DataBuffer 
{
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> 
    {
        Ok(&self.0)
    }

    fn consume(&mut self, amt: usize) 
    {
        self.0.drain(0..amt);
    }
}

impl DataBuffer 
{
    pub fn new() -> Self {
        Self(vec![])
    }

    pub fn with_size(size: impl Into<usize>) -> Self {
        Self(vec![0; size.into()])
    }
    
    /// Pop at more nb_bytes from the buffer and returns it in dedicated buffer
    pub fn pop_front(&mut self, nb_bytes: impl Into<usize>) -> DataBuffer 
    {
        let mut nb_bytes: usize  = nb_bytes.into();
        nb_bytes = min::<usize>(self.len(), nb_bytes);
        DataBuffer(self.0.drain(0..nb_bytes).collect())
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

    pub fn get_buf_write(&mut self) -> BufWriter<Cursor<&mut [u8]>> 
    {
        BufWriter::new(Cursor::new(&mut self.0))       
    }

    pub fn get_buf_read(&self) -> BufReader<Cursor<&[u8]>>
    {
        BufReader::new(Cursor::new(&self.0))
    }

    pub fn is_empty(&self) -> bool 
    {
        self.0.is_empty()
    }
}

impl OutStream for DataBuffer 
{
    fn write_to_stream<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<usize> {
        let mut data = self.0.clone();
        writer.write(&mut data)
    }

    fn write_all_to_stream<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        let mut data = self.0.clone();
        writer.write_all(&mut data)       
    }
}

impl InStream for DataBuffer {
    fn read_from_stream<R: std::io::BufRead>(&mut self, read: &mut R) -> std::io::Result<()> {
        read.read_exact(self)
    }
}
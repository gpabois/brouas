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

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct SizedRawData<const SIZE: usize>([u8; SIZE]);

impl<const SIZE: usize> Deref for SizedRawData<SIZE> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const SIZE: usize> DerefMut for SizedRawData<SIZE> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<const SIZE: usize> Into<DataBuffer> for SizedRawData<SIZE> 
{
    fn into(self) -> DataBuffer {
        DataBuffer(self.0.into())
    }
}

impl<const SIZE: usize> Default for SizedRawData<SIZE> {
    fn default() -> Self {
        Self::new_zeroed()
    }
}

impl<const SIZE: usize> SizedRawData<SIZE> {

    pub fn new_zeroed() -> Self {
        Self([0; SIZE])
    }

    pub fn new(data: [u8; SIZE]) -> Self {
        Self(data)
    }
}

impl<const SIZE: usize> InStream for SizedRawData<SIZE> 
{
    fn read_from_stream<R: std::io::BufRead>(&mut self, read: &mut R) -> std::io::Result<()> {
        read.read_exact(&mut self.0)?;
        Ok(())
    }
}

impl<const SIZE: usize> OutStream for SizedRawData<SIZE> {
    fn write_to_stream<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<usize> {
        writer.write(&self.0)
    }

    fn write_all_to_stream<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&self.0)
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

    pub fn with_size(size: usize) -> Self {
        Self(vec![0; size])
    }
    
    /// Pop at more nb_bytes from the buffer and returns it in dedicated buffer
    pub fn pop_front(&mut self, mut nb_bytes: u64) -> DataBuffer 
    {
        nb_bytes = min(self.len() as u64, nb_bytes);
        DataBuffer(self.0.drain(..nb_bytes as usize).collect())
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
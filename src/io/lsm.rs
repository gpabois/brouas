use std::{ops::Deref, io::{Cursor, Write}};

use crate::object::ObjectId;

use super::{traits::{WriteStream, OutStream, ReadStream, InStream, Writable}, Data};

pub struct Key(Vec<u8>);

impl From<&str> for Key {
    fn from(value: &str) -> Self {
        Self(Vec::from(value))
    }
}

impl From<ObjectId> for Key {
    fn from(key: ObjectId) -> Self {
        Self(Vec::from(key.to_le_bytes()))
    }
}

impl Deref for Key {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct Lsm(sled::Db);

impl Writable for Lsm {
    type Args = Key;
    type Write = LsmWrite;

    fn open_write(&mut self, args: Self::Args) -> std::io::Result<Self::Write> {
        todo!()
    }
}

pub struct LsmWrite<'key, 'lsm>(&'key[u8], Data, &'lsm mut sled::Db);

impl<'key, 'lsm> Write for LsmWrite<'key, 'lsm> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.1.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.2.insert(self.0, self.data.deref())
    }
}

impl<'key, 'lsm> Drop for LsmWrite<'key, 'lsm> {
    fn drop(&mut self) {
        self.flush().unwrap()
    }
}

pub mod traits {
    use crate::io::traits::{Input, Output};

    // State based stream (W/R)
    pub trait Storage {
        type Key;

        fn store<Out>(&mut self, key: &Self::Key, out: Out::OutputType) -> std::io::Result<()>
        where Out: Output;

        fn fetch<In>(&mut self, key: &Self::Key, input: &mut In::InputType) -> std::io::Result<()>
        where In: Input;
    }
}

use std::ops::{Deref};
use crate::{io::{traits::{Output, Input}, DataRef, Data}};

use self::traits::Storage;

#[derive(Clone)]
pub struct Key(Vec<u8>);

impl From<&str> for Key {
    fn from(value: &str) -> Self {
        Self(Vec::from(value))
    }
}

impl Deref for Key {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct Lsm {
    tree: sled::Db
}

impl Storage for Lsm {
    type Key = Key;

    fn store<Out>(&mut self, key: &Self::Key, out: Out::OutputType) -> std::io::Result<()>
    where Out: Output {
        let data = Data::new();
        Out::write_all_to(&out, &mut data);
        self.tree.insert(key.deref(), data.deref()).unwrap();
        Ok(())
    }

    fn fetch<In>(&mut self, key: &Self::Key, input: &mut In::InputType) -> std::io::Result<()>
    where In: Input {
        let raw = self.tree.get(key.deref()).unwrap().unwrap();

        Ok(())
    }
}

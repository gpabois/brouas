use std::{io::{Write, Read}, marker::PhantomData, any::TypeId};

use crate::io::{traits::{OutStream, InStream}, Data};

pub type ObjectId = u64;
pub type ObjectType = u16;
pub type ObjectSize = usize;


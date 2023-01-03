use super::error::Error;
use super::{ObjectId, ObjectType};
use super::result::Result;

#[derive(Copy, Clone)]
pub struct ObjectMeta {
    id: ObjectId,
    otype: ObjectType
}

impl ObjectMeta {
    pub fn new(id: ObjectId, otype: ObjectType) -> Self {
        Self{id, otype}
    }

    pub fn get_type(&self) -> ObjectType {
        self.otype
    }

    pub fn get_id(&self) -> ObjectId {
        self.id
    }

    pub fn assert_type(&self, otype: &ObjectType) -> Result<()> {
        if self.otype != *otype {
            return Err(Error::InvalidObjectType { expected: *otype, got: self.otype })
        }
        Ok(())
    }
}

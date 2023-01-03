use super::ObjectType;

pub enum Error {
    IO(std::io::Error),
    ObjectNotFound,
    InvalidObjectType{expected: ObjectType, got: ObjectType}
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::IO(err)
    }
}

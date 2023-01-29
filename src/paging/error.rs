
#[derive(Debug)]
pub enum Error {
    BufferError(crate::buffer::Error),
    IoError(std::io::Error)
}

impl Into<std::io::Error> for Error {
    fn into(self) -> std::io::Error {
        match self {
            Error::BufferError(err) => std::io::Error::new(std::io::ErrorKind::OutOfMemory, format!("memory buffer error: {:?}", err)),
            Error::IoError(err) => err,
        }
    }
}

impl From<crate::buffer::Error> for Error {
    fn from(err: crate::buffer::Error) -> Self {
        Self::BufferError(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::IoError(err)
    }
}
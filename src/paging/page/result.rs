use super::error::PageError;

pub type PageResult<T> = std::result::Result<T, PageError>;
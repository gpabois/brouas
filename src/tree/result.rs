use super::error::TreeError;

pub type TreeResult< T, Hash> = Result<T, TreeError< Hash>>;
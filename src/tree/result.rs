use super::error::TreeError;

pub type TreeResult<'a, T, Hash> = Result<T, TreeError<'a, Hash>>;
use super::{page_type::PageType, id::PageId};


#[derive(Debug)]
pub enum PagerError
{
    IOError(std::io::Error),
    WrongPageType{expecting: PageType, got: PageType},
    PageNotOpened(PageId),
    PageOverflow,
    PageFull(PageId),
    //
    SparseCell,
    OutOfBoundCell
}

impl From<std::io::Error> for PagerError {
    fn from(err: std::io::Error) -> Self {
        Self::IOError(err)
    }
}
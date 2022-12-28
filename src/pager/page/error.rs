use super::{id::PageId, page_type::PageType};

#[derive(Debug)]
pub enum PageError
{
    IOError(std::io::Error),
    WrongPageType{expecting: PageType, got: PageType},
    PageNotOpened(PageId),
    PageOverflow,
    PageFull(PageId)
}

impl From<std::io::Error> for PageError {
    fn from(err: std::io::Error) -> Self {
        Self::IOError(err)
    }
}


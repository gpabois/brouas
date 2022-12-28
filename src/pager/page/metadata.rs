use super::{nonce::PageNonce, id::PageId, page_type::PageType, header::PageHeader};

pub struct PageMetadata {
    /// Number of the page.
    pub id: PageId,
    /// Nonce, in case of conflicted pages.
    pub nonce: PageNonce,
    /// Type of page 
    pub page_type: PageType,
    /// Parent page
    pub parent_id: Option<PageId>
}

impl From<PageHeader> for PageMetadata {
    fn from(h: PageHeader) -> Self {
        Self {
            id: h.id,
            nonce: h.nonce,
            page_type: h.page_type,
            parent_id: h.parent_id
        }
    }
}
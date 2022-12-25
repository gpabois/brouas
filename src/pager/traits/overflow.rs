use crate::{io::{DataBuffer, traits::InStream}, pager::{id::PageId, result::PagerResult}};
use super::pager::Pager;

pub trait Overflow {
    fn write<P: Pager>(pager: &mut P, data: &mut DataBuffer, base: Option<PageId>) -> PagerResult<Option<PageId>>;
    fn read<P: Pager, E: InStream>(pager: &mut P, to: &mut E, page_id: &PageId, base: Option<&mut DataBuffer>) -> PagerResult<()>;
    fn read_and_instantiate<P: Pager, E: InStream + Default>(pager: &mut P, page_id: &PageId, base: Option<&mut DataBuffer>) -> PagerResult<E>;
}
use std::fmt::Display;

use crate::arena::traits::TLElementRef;

use super::NodeRef;

pub enum TreeError<Hash>
where Hash: Clone + PartialEq + Display
{
    ExpectingLeaf,
    MissingLeaf,
    MissingNode(NodeRef<Hash>)
}

impl<Hash> std::fmt::Debug for TreeError<Hash>
where Hash: Clone + PartialEq + Display
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ExpectingLeaf => write!(f, "Tree error: expecting leaf"),
            Self::MissingLeaf => write!(f, "Tree error: missing leaf"),
            Self::MissingNode(arg0) => {
                write!(f, "Tree error: missing node {}", arg0)
            }
        }
    }
}
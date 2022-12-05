use std::fmt::Display;

pub enum TreeError<'a, Node>
where Node: crate::tree::node::traits::Node<'a>
{
    BorrowMutError,
    ExpectingLeaf,
    ExpectingBranch,
    MissingLeaf,
    MissingNode(Node::Hash)
}

impl<'a, Node> std::fmt::Debug for TreeError<'a, Node>
where Node: crate::tree::node::traits::Node<'a>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ExpectingLeaf => write!(f, "tree error: expecting leaf"),
            Self::MissingLeaf => write!(f, "tree error: missing leaf"),
            Self::MissingNode(arg0) => {
                write!(f, "tree error: missing node {}", arg0)
            },
            Self::BorrowMutError => write!(f, "Tree error: cannot borrow mut node"),
            Self::ExpectingBranch => write!(f, "tree error: expecting branch")
        }
    }
}
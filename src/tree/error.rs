use std::fmt::Display;

pub enum TreeError<Node>
where Node: crate::tree::node::traits::Node
{
    ExpectingLeaf,
    MissingLeaf,
    MissingNode(Node::Hash)
}

impl<Node> std::fmt::Debug for TreeError<Node>
where Node: crate::tree::node::traits::Node
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
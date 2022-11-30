use super::NodeRef;

pub enum TreeError<Hash: Clone + PartialEq>
{
    ExpectingLeaf,
    MissingLeaf,
    MissingNode(NodeRef<Hash>)
}
use crate::bptree::nodes::BPTreeNodes;

pub fn nodes_fixture<K, V>() -> BPTreeNodes<K, V> {
    BPTreeNodes::new()
}
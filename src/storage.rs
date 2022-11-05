pub mod impls;

pub trait Storage<K, V> {
    fn save(&self, v: impl AsRef<V>) -> std::io::Result<()>;
    fn load(&self, k: impl AsRef<K>) -> std::io::Result<V>;
}

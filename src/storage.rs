pub mod impls;

pub trait Storage<K=crate::block::BlockHash> {
    type K;
    type V;

    /// Execute les sauvegardes en attente
    fn flush(&self) -> std::io::Result<()>;

    /// Sauvegarde l'entrée
    fn save(&self, v: impl AsRef<Self::V>) -> std::io::Result<()>;

    /// Charge une entrée
    fn load(&self, k: impl AsRef<Self::K>) -> std::io::Result<Self::V>;

    /// Vérifie si une entrée existe
    fn exist(&self, k: impl AsRef<Self::K>) -> bool;
}

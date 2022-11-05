use sha2::Sha256;
use sha2::Digest;

pub fn sha256(data: impl AsRef<[u8]>) -> [u8; 32]
{
    let mut recv: [u8; 32] = [0; 32];
    let mut hasher = Sha256::new();
    hasher.update(data.as_ref());
    recv.copy_from_slice(&hasher.finalize()[..]);
    return recv;
}

pub fn sha256_string(data: impl AsRef<[u8]>) -> String 
{
    hex::encode(sha256(data))
}
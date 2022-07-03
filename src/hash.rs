use bytes::Bytes;
use sha2::{Digest, Sha256};

pub fn hash_bytes(bytes: &Bytes) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let result = hasher.finalize();
    hex::encode(result)
}

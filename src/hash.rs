use serde::Serialize;
use sha2::{Digest, Sha256};

pub(crate) fn sha256_bytes(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

pub(crate) fn sha256_json<T: Serialize>(value: &T) -> String {
    let bytes = serde_json::to_vec(value).expect("serializing internal typed data cannot fail");
    sha256_bytes(&bytes)
}

use bytes::Bytes;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Metadata(serde_json::Value);

impl From<Bytes> for Metadata {
    fn from(bytes: Bytes) -> Self {
        Metadata(serde_json::from_slice(&bytes).expect("Error deserializing metadata"))
    }
}

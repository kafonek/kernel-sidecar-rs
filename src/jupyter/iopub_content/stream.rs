use bytes::Bytes;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
enum StreamName {
    Stdout,
    Stderr,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct Stream {
    name: StreamName,
    text: String,
}

impl From<Bytes> for Stream {
    fn from(bytes: Bytes) -> Self {
        serde_json::from_slice(&bytes).expect("Failed to deserialize Stream")
    }
}

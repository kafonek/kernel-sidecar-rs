/*
https://jupyter-client.readthedocs.io/en/latest/messaging.html#execution-errors
*/

use bytes::Bytes;
use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Clone, Serialize, PartialEq, Deserialize, Debug)]
pub struct Error {
    ename: String,
    evalue: String,
    traceback: Vec<String>,
}

impl From<Bytes> for Error {
    fn from(bytes: Bytes) -> Self {
        serde_json::from_slice(&bytes).expect("Failed to deserialize Error")
    }
}

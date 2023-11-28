/*
https://jupyter-client.readthedocs.io/en/latest/messaging.html#id6
*/
use std::collections::HashMap;

use bytes::Bytes;

use serde::Deserialize;

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct ExecuteResult {
    execution_count: u32,
    data: HashMap<String, serde_json::Value>,
    metadata: serde_json::Value,
}

impl From<Bytes> for ExecuteResult {
    fn from(bytes: Bytes) -> Self {
        serde_json::from_slice(&bytes).expect("Failed to deserialize ExecuteResult")
    }
}

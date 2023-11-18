use bytes::Bytes;
use serde::{Deserialize, Serialize};
// Status comes down over iopub for all messages

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum KernelStatus {
    Busy,
    Idle,
    Starting,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Status {
    pub execution_state: KernelStatus,
}

impl From<Bytes> for Status {
    fn from(bytes: Bytes) -> Self {
        serde_json::from_slice(&bytes).unwrap()
    }
}

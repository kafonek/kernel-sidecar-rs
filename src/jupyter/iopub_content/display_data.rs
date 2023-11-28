/*
https://jupyter-client.readthedocs.io/en/latest/messaging.html#display-data
*/
use std::collections::HashMap;

use bytes::Bytes;
use serde::Deserialize;

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct Transient {
    display_id: String,
}

// If the transient field is an empty dict, deserialize it as None
// otherwise deserialize it as Some(Transient)
fn deserialize_transient<'de, D>(deserializer: D) -> Result<Option<Transient>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let v: serde_json::Value = Deserialize::deserialize(deserializer)?;
    match v {
        serde_json::Value::Object(map) if map.is_empty() => Ok(None),
        _ => {
            let transient: Transient =
                serde_json::from_value(v).map_err(serde::de::Error::custom)?;
            Ok(Some(transient))
        }
    }
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct DisplayData {
    data: HashMap<String, serde_json::Value>,
    metadata: serde_json::Value,
    #[serde(deserialize_with = "deserialize_transient")]
    transient: Option<Transient>,
}

impl From<Bytes> for DisplayData {
    fn from(bytes: Bytes) -> Self {
        serde_json::from_slice(&bytes).expect("Failed to deserialize DisplayData")
    }
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct UpdateDisplayData {
    data: HashMap<String, serde_json::Value>,
    metadata: serde_json::Value,
    #[serde(deserialize_with = "deserialize_transient")]
    transient: Option<Transient>,
}

impl From<Bytes> for UpdateDisplayData {
    fn from(bytes: Bytes) -> Self {
        serde_json::from_slice(&bytes).expect("Failed to deserialize UpdateDisplayData")
    }
}

/*
Models a Notebook document. https://ipython.org/ipython-doc/3/notebook/nbformat.html
*/

use serde::{Deserialize, Deserializer, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Notebook {
    pub cells: Vec<Cell>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    pub metadata: serde_json::Value,
    pub nbformat: u32,
    pub nbformat_minor: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "output_type", rename_all = "snake_case")]
pub enum Output {
    // TODO: look into using the content structs from jupyter/iopub_content instead of redefining?
    DisplayData(serde_json::Value),
    Stream {
        name: String,
        #[serde(deserialize_with = "list_or_string_to_string")]
        text: String,
    },
    ExecuteResult {
        execution_count: u32,
        data: serde_json::Value,
        metadata: serde_json::Value,
    },
    Error {
        ename: String,
        evalue: String,
        traceback: Vec<String>,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "cell_type", rename_all = "lowercase")]
pub enum Cell {
    Code(CodeCell),
    Markdown(MarkdownCell),
    Raw(RawCell),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CodeCell {
    pub id: String,
    #[serde(deserialize_with = "list_or_string_to_string")]
    pub source: String,
    pub metadata: serde_json::Value,
    pub execution_count: Option<u32>,
    pub outputs: Vec<Output>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MarkdownCell {
    pub id: String,
    #[serde(deserialize_with = "list_or_string_to_string")]
    pub source: String,
    pub metadata: serde_json::Value,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct RawCell {
    pub id: String,
    #[serde(deserialize_with = "list_or_string_to_string")]
    pub source: String,
    pub metadata: serde_json::Value,
}

// Custom deserialization for source field since it may be a Vec<String> or String
fn list_or_string_to_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    // Deserialize the source field as a serde_json::Value
    let source_value: serde_json::Value = Deserialize::deserialize(deserializer)?;

    // Check if the source is an array of strings
    if let Some(source_array) = source_value.as_array() {
        // Join the array of strings into a single string
        let source_string = source_array
            .iter()
            .map(|s| s.as_str().unwrap_or_default())
            .collect::<Vec<_>>()
            .join("\n");

        Ok(source_string)
    } else if let Some(source_str) = source_value.as_str() {
        // If source is already a string, return it
        Ok(source_str.to_string())
    } else {
        Err(serde::de::Error::custom("Invalid source format"))
    }
}
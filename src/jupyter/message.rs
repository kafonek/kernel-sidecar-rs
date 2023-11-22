use crate::jupyter::header::Header;
use crate::jupyter::metadata::Metadata;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Message<T> {
    pub header: Header,
    pub parent_header: Option<Header>,
    pub metadata: Option<Metadata>,
    pub content: T,
}

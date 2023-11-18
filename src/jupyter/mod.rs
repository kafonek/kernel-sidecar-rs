pub mod header;
pub mod messages;
pub mod metadata;
pub mod wire_protocol;

use header::Header;
use messages::kernel_info::{KernelInfoReply, KernelInfoRequest};
use messages::status::Status;
use metadata::Metadata;
use serde::{Deserialize, Serialize};

use wire_protocol::WireProtocol;
use zeromq::ZmqMessage;

use self::messages::execute::ExecuteRequest;

#[derive(Debug, Serialize, Deserialize)]
pub struct Message<T> {
    pub header: Header,
    pub parent_header: Option<Header>,
    pub metadata: Option<Metadata>,
    pub content: T,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UnmodeledContent(serde_json::Value);

#[derive(Debug)]
pub enum Request {
    KernelInfo(Message<KernelInfoRequest>),
    Execute(Message<ExecuteRequest>),
}

impl Request {
    pub fn msg_id(&self) -> String {
        // return msg_id from header
        match self {
            Request::KernelInfo(msg) => msg.header.msg_id.to_owned(),
            Request::Execute(msg) => msg.header.msg_id.to_owned(),
        }
    }

    pub fn into_wire_protocol(&self, hmac_signing_key: &str) -> WireProtocol {
        match self {
            Request::KernelInfo(msg) => WireProtocol::new(
                msg.header.clone(),
                Some(msg.content.clone()),
                hmac_signing_key,
            ),
            Request::Execute(msg) => WireProtocol::new(
                msg.header.clone(),
                Some(msg.content.clone()),
                hmac_signing_key,
            ),
        }
    }
}

#[derive(Debug)]
pub enum Response {
    Status(Message<Status>),
    KernelInfo(Message<KernelInfoReply>),
    Unmodeled(Message<UnmodeledContent>),
}

impl Response {
    pub fn msg_id(&self) -> String {
        // return msg_id from parent header
        match self {
            Response::Status(msg) => msg.parent_header.as_ref().unwrap().msg_id.to_owned(),
            Response::KernelInfo(msg) => msg.parent_header.as_ref().unwrap().msg_id.to_owned(),
            Response::Unmodeled(msg) => msg.parent_header.as_ref().unwrap().msg_id.to_owned(),
        }
    }
}

impl From<WireProtocol> for Response {
    fn from(wp: WireProtocol) -> Self {
        let header: Header = wp.header.into();
        let parent_header: Header = wp.parent_header.into();
        let metadata: Metadata = wp.metadata.into();
        match header.msg_type.as_str() {
            "status" => {
                let content: Status = wp.content.into();
                let msg: Message<Status> = Message {
                    header,
                    parent_header: Some(parent_header),
                    metadata: Some(metadata),
                    content,
                };
                Response::Status(msg)
            }
            "kernel_info_reply" => {
                let content: KernelInfoReply = wp.content.into();
                let msg: Message<KernelInfoReply> = Message {
                    header,
                    parent_header: Some(parent_header),
                    metadata: Some(metadata),
                    content,
                };
                Response::KernelInfo(msg)
            }
            _ => {
                let content: UnmodeledContent = serde_json::from_slice(&wp.content)
                    .expect("Error deserializing unmodeled content");
                let msg: Message<UnmodeledContent> = Message {
                    header,
                    parent_header: Some(parent_header),
                    metadata: Some(metadata),
                    content,
                };
                Response::Unmodeled(msg)
            }
        }
    }
}

impl From<ZmqMessage> for Response {
    fn from(msg: ZmqMessage) -> Self {
        let wp: WireProtocol = msg.into();
        wp.into()
    }
}

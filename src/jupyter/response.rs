use crate::jupyter::header::Header;
use crate::jupyter::message::Message;
use crate::jupyter::message_content::kernel_info::KernelInfoReply;
use crate::jupyter::message_content::status::Status;
use crate::jupyter::metadata::Metadata;
use crate::jupyter::wire_protocol::WireProtocol;
use serde::{Deserialize, Serialize};

use zeromq::ZmqMessage;

#[derive(Debug, Serialize, Deserialize)]
pub struct UnmodeledContent(serde_json::Value);

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

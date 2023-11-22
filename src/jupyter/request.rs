use crate::jupyter::message::Message;
use crate::jupyter::messages::execute::ExecuteRequest;
use crate::jupyter::messages::kernel_info::KernelInfoRequest;
use crate::jupyter::wire_protocol::WireProtocol;

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

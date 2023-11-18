use std::{fmt::Debug, sync::Arc};

use tokio::sync::Mutex;

use crate::jupyter::{messages::status::KernelStatus, Request, Response};

#[async_trait::async_trait]
pub trait Handler: Debug + Send + Sync {
    async fn handle(&self, msg: &Response);
}

#[derive(Debug, PartialEq)]
pub enum ExpectedReplyType {
    KernelInfo,
    None,
}

impl From<&Request> for ExpectedReplyType {
    fn from(request: &Request) -> Self {
        match request {
            Request::KernelInfo(_) => ExpectedReplyType::KernelInfo,
            _ => ExpectedReplyType::None,
        }
    }
}

impl From<&Response> for ExpectedReplyType {
    fn from(response: &Response) -> Self {
        match response {
            Response::KernelInfo(_) => ExpectedReplyType::KernelInfo,
            _ => ExpectedReplyType::None,
        }
    }
}

#[derive(Debug)]
struct ActionState {
    kernel_idle: Arc<Mutex<bool>>,
    expected_reply_seen: Arc<Mutex<bool>>,
    expected_reply_type: Option<ExpectedReplyType>,
}

#[derive(Debug)]
pub struct Action {
    handlers: Vec<Box<dyn Handler>>,
    state: ActionState,
    pub request: Request,
}

impl Action {
    pub fn new(request: Request, handlers: Vec<Box<dyn Handler>>) -> Self {
        let expected_reply_type = match ExpectedReplyType::from(&request) {
            ExpectedReplyType::None => None,
            expected_reply_type => Some(expected_reply_type),
        };
        let action_state = match expected_reply_type {
            Some(expected_reply_type) => ActionState {
                kernel_idle: Arc::new(Mutex::new(false)),
                expected_reply_seen: Arc::new(Mutex::new(false)),
                expected_reply_type: Some(expected_reply_type),
            },
            None => ActionState {
                kernel_idle: Arc::new(Mutex::new(false)),
                expected_reply_seen: Arc::new(Mutex::new(true)),
                expected_reply_type: None,
            },
        };
        Action {
            handlers,
            state: action_state,
            request,
        }
    }

    pub async fn handle(&self, msg: Response) {
        for handler in &self.handlers {
            handler.handle(&msg).await;
        }
        match msg {
            Response::Status(status) => {
                if status.content.execution_state == KernelStatus::Idle {
                    self.set_idle_seen().await;
                }
            }
            _ => {
                if self.state.expected_reply_type == Some(ExpectedReplyType::from(&msg)) {
                    self.set_expected_reply_seen().await;
                }
            }
        }
    }

    async fn set_idle_seen(&self) {
        let mut kernel_idle = self.state.kernel_idle.lock().await;
        *kernel_idle = true;
        if *kernel_idle && *self.state.expected_reply_seen.lock().await {
            dbg!("kernel_idle and expected_reply_seen");
        }
    }

    async fn set_expected_reply_seen(&self) {
        let mut expected_reply_seen = self.state.expected_reply_seen.lock().await;
        *expected_reply_seen = true;
        if *expected_reply_seen && *self.state.kernel_idle.lock().await {
            dbg!("kernel_idle and expected_reply_seen");
        }
    }
}

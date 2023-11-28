use tokio::sync::Mutex;

use crate::jupyter::response::Response;
use std::{collections::HashMap, fmt::Debug, sync::Arc};

#[async_trait::async_trait]
pub trait Handler: Debug + Send + Sync {
    async fn handle(&self, msg: &Response);
}

// dbg!'s all messages handled by an Action
#[derive(Debug)]
pub struct DebugHandler;

impl DebugHandler {
    pub fn new() -> Self {
        DebugHandler {}
    }
}

#[async_trait::async_trait]
impl Handler for DebugHandler {
    async fn handle(&self, msg: &Response) {
        dbg!(msg);
    }
}

// Returns a hashmap of {msg_type: count} for all messages handled by an Action
#[derive(Debug, Clone)]
pub struct MessageCountHandler {
    pub counts: Arc<Mutex<HashMap<String, usize>>>,
}

impl MessageCountHandler {
    pub fn new() -> Self {
        MessageCountHandler {
            counts: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait::async_trait]
impl Handler for MessageCountHandler {
    async fn handle(&self, msg: &Response) {
        let mut counts = self.counts.lock().await;
        let msg_type = msg.msg_type();
        let count = counts.entry(msg_type).or_insert(0);
        *count += 1;
    }
}

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::Mutex;

use crate::jupyter::response::Response;

use crate::handlers::Handler;

// Returns a hashmap of {msg_type: count} for all messages handled by an Action
// Primarily used in tests and introspective click-testing
#[derive(Debug, Clone)]
pub struct MessageCountHandler {
    pub counts: Arc<Mutex<HashMap<String, usize>>>,
}

impl Default for MessageCountHandler {
    fn default() -> Self {
        Self::new()
    }
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

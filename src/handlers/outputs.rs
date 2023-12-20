use crate::handlers::Handler;
use crate::jupyter::response::Response;
use std::collections::HashMap;
use std::fmt::Debug;

#[async_trait::async_trait]
pub trait OutputHandler: Handler + Debug + Send + Sync {
    async fn add_cell_content(&self, content: &HashMap<String, serde_json::Value>);
    async fn clear_cell_content(&self);

    async fn handle_output(&self, msg: &Response) {
        match msg {
            Response::ExecuteResult(result) => {
                self.add_cell_content(&result.content.data).await;
            }
            _ => {}
        }
    }
}

// Need this here so that structs can impl OutputHandler and not get yelled
// at about also needing to impl Handler
#[async_trait::async_trait]
impl<T: OutputHandler + Send + Sync> Handler for T {
    async fn handle(&self, msg: &Response) {
        self.handle_output(msg).await;
    }
}

use crate::handlers::Handler;
use crate::jupyter::response::Response;
use crate::notebook::Output;

use std::fmt::Debug;

#[async_trait::async_trait]
pub trait OutputHandler: Handler + Debug + Send + Sync {
    async fn add_cell_content(&self, content: Output);
    async fn clear_cell_content(&self);

    async fn handle_output(&self, msg: &Response) {
        match msg {
            Response::ExecuteResult(m) => {
                let output = Output::ExecuteResult(m.content.clone());
                self.add_cell_content(output).await;
            }
            Response::Stream(m) => {
                let output = Output::Stream(m.content.clone());
                self.add_cell_content(output).await;
            }
            Response::ClearOutput(_m) => {
                self.clear_cell_content().await;
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

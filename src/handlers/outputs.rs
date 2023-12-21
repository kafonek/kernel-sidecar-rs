use tokio::sync::RwLock;

use crate::handlers::Handler;
use crate::jupyter::response::Response;
use crate::notebook::Output;

use std::fmt::Debug;
use std::sync::Arc;

#[async_trait::async_trait]
pub trait OutputHandler: Handler + Debug + Send + Sync {
    // expect a struct implementing this to have a .clear_on_next_output bool attribute
    async fn add_cell_content(&self, content: Output);
    async fn clear_cell_content(&self);
    async fn sync_display_data(&self, content: Output);

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
            Response::DisplayData(m) => {
                let output = Output::DisplayData(m.content.clone());
                self.add_cell_content(output).await;
            }
            Response::Error(m) => {
                let output = Output::Error(m.content.clone());
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

#[derive(Debug, Clone)]
pub struct SimpleOutputHandler {
    pub output: Arc<RwLock<Vec<Output>>>,
}

impl Default for SimpleOutputHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl SimpleOutputHandler {
    pub fn new() -> Self {
        Self {
            output: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

#[async_trait::async_trait]
impl OutputHandler for SimpleOutputHandler {
    async fn add_cell_content(&self, content: Output) {
        self.output.write().await.push(content);
        println!("add_cell_content");
    }

    async fn clear_cell_content(&self) {
        self.output.write().await.clear();
        println!("clear_cell_content");
    }

    async fn sync_display_data(&self, _content: Output) {
        // Don't do anything for sync display data in SimpleOutputHandler
        // we aren't keeping any reference to the full Notebook document in order to
        // update the display data by id outside of this "current cell" context
    }
}

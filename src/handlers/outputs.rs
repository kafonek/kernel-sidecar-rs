use tokio::sync::{Mutex, RwLock};

use crate::handlers::Handler;
use crate::jupyter::response::Response;
use crate::notebook::Output;

use std::fmt::Debug;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct SimpleOutputHandler {
    // interior mutability here because .handle needs to set this and is &self, and when trying
    // to change that to &mut self then it broke the delegation of ZMQ messages to Actions over
    // in actions.rs. TODO: come back to this when I'm better at Rust?
    clear_on_next_output: Arc<Mutex<bool>>,
    pub output: Arc<RwLock<Vec<Output>>>,
}

impl SimpleOutputHandler {
    pub fn new() -> Self {
        Self {
            clear_on_next_output: Arc::new(Mutex::new(false)),
            output: Arc::new(RwLock::new(vec![])),
        }
    }

    async fn add_cell_content(&self, content: Output) {
        self.output.write().await.push(content);
        println!("add_cell_content");
    }

    async fn clear_cell_content(&self) {
        self.output.write().await.clear();
        println!("clear_cell_content");
    }
}

#[async_trait::async_trait]
impl Handler for SimpleOutputHandler {
    async fn handle(&self, msg: &Response) {
        let mut clear_on_next_output = self.clear_on_next_output.lock().await;
        match msg {
            Response::ExecuteResult(m) => {
                let output = Output::ExecuteResult(m.content.clone());
                if *clear_on_next_output {
                    self.clear_cell_content().await;
                    *clear_on_next_output = false;
                }
                self.add_cell_content(output).await;
            }
            Response::Stream(m) => {
                let output = Output::Stream(m.content.clone());
                if *clear_on_next_output {
                    self.clear_cell_content().await;
                    *clear_on_next_output = false;
                }
                self.add_cell_content(output).await;
            }
            Response::DisplayData(m) => {
                let output = Output::DisplayData(m.content.clone());
                if *clear_on_next_output {
                    self.clear_cell_content().await;
                    *clear_on_next_output = false;
                }
                self.add_cell_content(output).await;
            }
            Response::Error(m) => {
                let output = Output::Error(m.content.clone());
                if *clear_on_next_output {
                    self.clear_cell_content().await;
                    *clear_on_next_output = false;
                }
                self.add_cell_content(output).await;
            }
            Response::ClearOutput(m) => {
                if m.content.wait {
                    *clear_on_next_output = true;
                } else {
                    self.clear_cell_content().await;
                }
            }
            _ => {}
        }
    }
}

use std::sync::Arc;

use tokio::sync::Mutex;

use crate::{
    handlers::Handler,
    jupyter::response::Response,
    notebook::{Cell, CodeCell, MarkdownCell, Notebook, Output},
};

#[derive(Debug, Clone)]
pub struct NotebookBuilder {
    pub nb: Notebook,
}

impl NotebookBuilder {
    pub fn new() -> Self {
        Self {
            nb: Notebook::new(),
        }
    }

    pub fn from_file(filename: &str) -> Self {
        Self {
            nb: Notebook::from_file(filename),
        }
    }

    pub fn save(&self, filename: &str) {
        self.nb.save(filename);
    }

    pub fn get_cell(&self, id: &str) -> Option<&Cell> {
        for cell in &self.nb.cells {
            if cell.id() == id {
                return Some(cell);
            }
        }
        None
    }

    pub fn get_cell_mut(&mut self, id: &str) -> Option<&mut Cell> {
        for cell in &mut self.nb.cells {
            if cell.id() == id {
                return Some(cell);
            }
        }
        None
    }

    pub fn add_cell(&mut self, cell: Cell) {
        self.nb.cells.push(cell);
    }

    pub fn add_code_cell(&mut self, source: &str) -> Cell {
        let cell = Cell::Code(CodeCell {
            id: uuid::Uuid::new_v4().to_string(),
            source: source.to_owned(),
            metadata: serde_json::Value::Null,
            execution_count: None,
            outputs: vec![],
        });
        self.add_cell(cell.clone());
        cell
    }

    pub fn add_markdown_cell(&mut self, source: &str) -> Cell {
        let cell = Cell::Markdown(MarkdownCell {
            id: uuid::Uuid::new_v4().to_string(),
            source: source.to_owned(),
            metadata: serde_json::Value::Null,
        });
        self.add_cell(cell.clone());
        cell
    }

    pub fn clear_cell_output(&mut self, id: &str) {
        if let Some(cell) = self.get_cell_mut(id) {
            match cell {
                Cell::Code(cell) => cell.clear_output(),
                _ => {}
            }
        }
    }

    pub fn add_cell_output(&mut self, id: &str, output: Output) {
        if let Some(cell) = self.get_cell_mut(id) {
            match cell {
                Cell::Code(cell) => cell.outputs.push(output),
                _ => {}
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct BuilderOutputHandler {
    pub builder: Arc<Mutex<NotebookBuilder>>,
    pub cell_id: String,
}

impl BuilderOutputHandler {
    pub fn new(builder: Arc<Mutex<NotebookBuilder>>, cell_id: &str) -> Self {
        Self {
            builder: builder,
            cell_id: cell_id.to_owned(),
        }
    }

    async fn add_cell_content(&self, content: Output) {
        self.builder
            .lock()
            .await
            .add_cell_output(&self.cell_id, content);
    }

    async fn clear_cell_content(&self) {
        self.builder.lock().await.clear_cell_output(&self.cell_id);
    }
}

#[async_trait::async_trait]
impl Handler for BuilderOutputHandler {
    async fn handle(&self, msg: &Response) {
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
            Response::ClearOutput(_) => {
                self.clear_cell_content().await;
            }
            _ => {}
        }
    }
}

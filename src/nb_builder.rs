use std::sync::Arc;

use tokio::sync::RwLock;

use crate::handlers::Handler;
use crate::jupyter::response::Response;
use crate::notebook::{Cell, CodeCell, MarkdownCell, Notebook, Output};

#[derive(Debug, Clone)]
pub struct NotebookBuilder {
    nb: Arc<RwLock<Notebook>>,
}

impl Default for NotebookBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl NotebookBuilder {
    pub fn new() -> Self {
        Self {
            nb: Arc::new(RwLock::new(Notebook::new())),
        }
    }

    pub fn from_file(filename: &str) -> Self {
        Self {
            nb: Arc::new(RwLock::new(Notebook::from_file(filename))),
        }
    }

    pub fn output_handler(&self, cell_id: &str) -> OutputHandler {
        OutputHandler::new(self.clone(), cell_id)
    }

    pub async fn save(&self, filename: &str) {
        self.nb.read().await.save(filename);
    }

    pub async fn get_cell(&self, id: &str) -> Option<Cell> {
        let nb = self.nb.read().await;
        for cell in nb.cells.iter() {
            if cell.id() == id {
                return Some(cell.clone());
            }
        }
        None
    }

    pub async fn replace_cell(&self, cell: Cell) {
        let mut nb = self.nb.write().await;
        for i in 0..nb.cells.len() {
            if nb.cells[i].id() == cell.id() {
                nb.cells[i] = cell;
                return;
            }
        }
    }

    pub async fn add_cell(&self, cell: Cell) {
        let mut nb = self.nb.write().await;
        nb.cells.push(cell);
    }

    pub async fn add_code_cell(&self, source: &str) -> Cell {
        let cell = Cell::Code(CodeCell {
            id: uuid::Uuid::new_v4().to_string(),
            source: source.to_owned(),
            metadata: serde_json::Value::Null,
            execution_count: None,
            outputs: vec![],
        });
        self.add_cell(cell.clone()).await;
        cell
    }

    pub async fn add_markdown_cell(&self, source: &str) -> Cell {
        let cell = Cell::Markdown(MarkdownCell {
            id: uuid::Uuid::new_v4().to_string(),
            source: source.to_owned(),
            metadata: serde_json::Value::Null,
        });
        self.add_cell(cell.clone()).await;
        cell
    }

    pub async fn clear_cell_output(&self, id: &str) {
        // Get a (cloned) Cell from the Notebook, clear its output, and replace the existing cell
        // in the owned Notebook with the modified cleared-output Cell
        let cell = self.get_cell(id).await;
        if let Some(Cell::Code(mut cell)) = cell {
            cell.clear_output();
            self.replace_cell(Cell::Code(cell)).await;
        }
    }

    pub async fn add_cell_output(&self, id: &str, output: Output) {
        // Get a (cloned) Cell from the Notebook, add the output, and replace the existing cell
        // in the owned Notebook with the modified output Cell
        let cell = self.get_cell(id).await;
        if let Some(Cell::Code(mut cell)) = cell {
            cell.outputs.push(output);
            self.replace_cell(Cell::Code(cell)).await;
        }
    }
}

#[derive(Debug, Clone)]
pub struct OutputHandler {
    builder: NotebookBuilder,
    cell_id: String,
}

impl OutputHandler {
    pub fn new(builder: NotebookBuilder, cell_id: &str) -> Self {
        Self {
            builder,
            cell_id: cell_id.to_owned(),
        }
    }

    async fn add_cell_content(&self, content: Output) {
        self.builder.add_cell_output(&self.cell_id, content).await;
    }

    async fn clear_cell_content(&self) {
        self.builder.clear_cell_output(&self.cell_id).await;
    }
}

#[async_trait::async_trait]
impl Handler for OutputHandler {
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

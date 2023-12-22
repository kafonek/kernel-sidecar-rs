use kernel_sidecar_rs::client::Client;
use kernel_sidecar_rs::handlers::{DebugHandler, Handler, MessageCountHandler};
use kernel_sidecar_rs::kernels::JupyterKernel;
use kernel_sidecar_rs::nb_builder::NotebookBuilder;
use tokio::signal::unix::{signal, SignalKind};

use tokio::time::sleep;

use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() {
    // Effectively create an in-memory Notebook, not reading from disk
    let builder = NotebookBuilder::new();

    // Start ipykernel child process
    let silent = true;
    let kernel = JupyterKernel::ipython(silent);

    // Start ZMQ connections
    let client = Client::new(kernel.connection_info.clone()).await;
    client.heartbeat().await;
    // small sleep to make sure iopub is connected,
    sleep(Duration::from_millis(50)).await;

    // Add a new cell to the Notebook, it will make a random cell id. Returns Cell object
    let cell = builder.add_code_cell("2 + 3").await;

    let debug_handler = DebugHandler::new(); // Just for debug, prints out all ZMQ messages
    let msg_count_handler = MessageCountHandler::new(); // Just for debug, prints count of msg type
    let output_handler = builder.output_handler(cell.id()); // Updates in-memory Notebook with cell output
    let handlers = vec![
        Arc::new(debug_handler) as Arc<dyn Handler>,
        Arc::new(msg_count_handler.clone()) as Arc<dyn Handler>,
        Arc::new(output_handler) as Arc<dyn Handler>,
    ];

    // Send the cell source code over as an execute request.
    // The BuilderOutputHandler will update the in-memory Notebook with cell output
    let action = client.execute_request(cell.source(), handlers).await;

    let mut sigint = signal(SignalKind::interrupt()).expect("Failed to set up signal handler");

    tokio::select! {
        _ = action => {
            println!("action completed");
        }
        _ = sigint.recv() => {
            println!("SIGINT received");
        }
    }
    println!("Message counts: {:?}", msg_count_handler.counts);
    // Print out in-memory Notebook cell output
    println!("Cell: {:?}", builder.get_cell(cell.id()).await);
    // Save in-memory Notebook to disk
    builder.save("test.ipynb").await;
}

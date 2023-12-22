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
    // Create a blank in-memory Notebook, not reading existing file from disk
    let builder = NotebookBuilder::new();

    // Start ipykernel child process
    let silent = true;
    let kernel = JupyterKernel::ipython(silent);

    // Start ZMQ connections
    let client = Client::new(kernel.connection_info.clone()).await;
    client.heartbeat().await;
    // small sleep to make sure iopub is connected,
    sleep(Duration::from_millis(50)).await;

    // Add a new cell to the Notebook. Assigns random cell id. Returns cloned Cell object.
    // If thinking ahead towards CRDT's, could think of this as "dirty" (not synced to others)
    // but we're only using it to send source code in execute request, no big deal.
    let cell = builder.add_code_cell("2 + 3").await;

    // Just for debug, prints out all ZMQ messages
    let debug_handler = DebugHandler::new();
    // Just for debug, prints count of msg types at the end of script
    let msg_count_handler = MessageCountHandler::new();
    // Updates in-memory builder Notebook with cell output
    let output_handler = builder.output_handler(cell.id());
    let handlers = vec![
        Arc::new(debug_handler) as Arc<dyn Handler>,
        Arc::new(msg_count_handler.clone()) as Arc<dyn Handler>,
        Arc::new(output_handler) as Arc<dyn Handler>,
    ];

    // Send the cell source code over as an execute request, every ZMQ response gets processed
    // by all three handlers sequentially
    let action = client.execute_request(cell.source(), handlers).await;

    // Signal handling to support ctrl-c in the off chance something goes wrong and this script
    // never completes (missing expected shell/iopub messages for status or execute_reply?)
    let mut sigint = signal(SignalKind::interrupt()).expect("Failed to set up signal handler");

    tokio::select! {
        _ = action => {
            println!("action completed");
        }
        _ = sigint.recv() => {
            println!("SIGINT received");
        }
    }
    // Debug: print count of ZMQ response message types
    println!("Message counts: {:?}", msg_count_handler.counts);
    // Print out in-memory Notebook cell (source and outputs)
    println!("Cell: {:?}", builder.get_cell(cell.id()).await);
    // See what it looks like when saving in-memory Notebook to disk (serde for serialization)
    builder.save("test.ipynb").await;
}

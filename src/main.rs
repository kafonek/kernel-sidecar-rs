use kernel_sidecar_rs::client::Client;
use kernel_sidecar_rs::handlers::{DebugHandler, Handler, MessageCountHandler};
use kernel_sidecar_rs::kernels::JupyterKernel;
use kernel_sidecar_rs::nb_builder::{BuilderOutputHandler, NotebookBuilder};
use tokio::{
    signal::unix::{signal, SignalKind},
    sync::Mutex,
};

use tokio::time::sleep;

use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() {
    // Effectively create an in-memory Notebook, not reading from disk
    let builder = Arc::new(Mutex::new(NotebookBuilder::new()));

    let silent = true;
    let kernel = JupyterKernel::ipython(silent);

    // start the Kernel
    let client = Client::new(kernel.connection_info.clone()).await;
    client.heartbeat().await;
    // small sleep to make sure iopub is connected,
    sleep(Duration::from_millis(50)).await;

    // Add a new cell to the Notebook, it will make a random cell id. Returns Cell object
    let cell = builder.lock().await.add_code_cell("print('Hello World!')");

    // let debug_handler = DebugHandler::new();
    let msg_count_handler = MessageCountHandler::new();
    let output_handler = BuilderOutputHandler::new(builder.clone(), cell.id());
    let handlers = vec![
        //Arc::new(debug_handler) as Arc<dyn Handler>,
        Arc::new(msg_count_handler.clone()) as Arc<dyn Handler>,
        Arc::new(output_handler.clone()) as Arc<dyn Handler>,
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
    println!("Cell: {:?}", builder.lock().await.get_cell(cell.id()));
    // Save in-memory Notebook to disk
    builder.lock().await.save("test.ipynb");
}

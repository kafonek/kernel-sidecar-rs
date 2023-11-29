/*
Mainly here for click-testing various commands and code blocks for different Kernel backends.
*/
use kernel_sidecar_rs::client::Client;
use kernel_sidecar_rs::handlers::{DebugHandler, Handler, MessageCountHandler};
use kernel_sidecar_rs::kernels::JupyterKernel;
use tokio::signal::unix::{signal, SignalKind};
use tokio::time::sleep;

use indoc::indoc;
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() {
    let silent = true;
    let kernel = JupyterKernel::ipython(silent);
    let client = Client::new(kernel.connection_info.clone()).await;
    client.heartbeat().await;
    // small sleep to make sure iopub is connected,
    sleep(Duration::from_millis(50)).await;

    let debug_handler = DebugHandler::new();
    let msg_count_handler = MessageCountHandler::new();
    let handlers = vec![
        Arc::new(debug_handler) as Arc<dyn Handler>,
        Arc::new(msg_count_handler.clone()) as Arc<dyn Handler>,
    ];
    // let action = client.kernel_info_request(handlers).await;
    let code = indoc! {"2 + "};
    let action = client.execute_request(code.to_owned(), handlers).await;

    // Set up signal handling so that if awaiting the action hangs or there's a panic then if we
    // ctrl-c to get out, the child Kernel process is cleaned up from JupyterKernel::Drop
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
}

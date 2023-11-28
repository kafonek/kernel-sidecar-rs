use kernel_sidecar_rs::client::Client;
use kernel_sidecar_rs::handlers::{DebugHandler, Handler};
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

    let handler = DebugHandler::new();
    let handlers = vec![Arc::new(handler) as Arc<dyn Handler>];
    // let action = client.kernel_info_request(handlers).await;
    let code = indoc! {"
    from IPython.display import display
    display('hello world', display_id='123')
    "
    };
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
}

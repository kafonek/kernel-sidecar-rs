use kernel_sidecar_rs::client::Client;
use kernel_sidecar_rs::handlers::{DebugHandler, Handler};
use kernel_sidecar_rs::kernels::JupyterKernel;
use tokio::time::sleep;

use std::{sync::Arc, time::Duration};

#[tokio::main]
async fn main() {
    let kernel = JupyterKernel::deno(false);
    let client = Client::new(kernel.connection_info.clone()).await;
    client.heartbeat().await;
    // small sleep to make sure iopub is connected,
    sleep(Duration::from_millis(50)).await;

    let handler = DebugHandler::new();
    let handlers = vec![Arc::new(handler) as Arc<dyn Handler>];
    let action = client.kernel_info_request(handlers).await;
    // let action = client.execute_request("2 + 2".to_owned(), handlers).await;
    action.await;
}

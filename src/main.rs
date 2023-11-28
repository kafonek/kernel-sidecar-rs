use kernel_sidecar_rs::handlers::{DebugHandler, Handler};
use kernel_sidecar_rs::utils::JupyterKernel;
use kernel_sidecar_rs::{client::Client, jupyter::connection_file::ConnectionInfo};

use std::sync::Arc;

#[tokio::main]
async fn main() {
    let kernel = JupyterKernel::evcxr(false);
    let client = Client::new(kernel.connection_info.clone()).await;
    client.heartbeat().await;

    let handler = DebugHandler::new();
    let handlers = vec![Arc::new(handler) as Arc<dyn Handler>];
    // let action = client.kernel_info_request(handlers).await;
    let action = client
        .execute_request("println!(\"hello world\")".to_owned(), handlers)
        .await;
    action.await;
}

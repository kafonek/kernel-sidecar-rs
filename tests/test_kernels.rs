use std::collections::HashMap;

use std::sync::Arc;

use kernel_sidecar_rs::client::Client;
use kernel_sidecar_rs::handlers::{Handler, MessageCountHandler};
use kernel_sidecar_rs::kernels::JupyterKernel;

// Start Kernel (type based on feature flags) and wait for ZMQ channels to come up
async fn start_kernel() -> (JupyterKernel, Client) {
    let silent = true;
    let kernel = if cfg!(feature = "test_evcxr") {
        JupyterKernel::evcxr(silent)
    } else if cfg!(feature = "test_irkernel") {
        JupyterKernel::irkernel(silent)
    } else {
        JupyterKernel::ipython(silent)
    };
    let client = Client::new(kernel.connection_info.clone()).await;
    client.heartbeat().await;
    // Anecdotally, have noticed tests fail becaues Status messages aren't showing up as expected.
    // Theory is that heartbeat is returning but iopub isn't pushing out messages even though
    // shell is connected and accepting request / replies?
    // Could be totally wrong.
    // Separately, there may be an edge case where multiple JupyterKernel::ipython calls end up
    // with the same ports and it all blows up. TODO: fix that.
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    (kernel, client)
}

#[tokio::test]
async fn test_kernel_info() {
    let (_kernel, client) = start_kernel().await;

    // send kernel_info_request
    let handler = MessageCountHandler::new();
    let handlers = vec![Arc::new(handler.clone()) as Arc<dyn Handler>];
    let action = client.kernel_info_request(handlers).await;
    action.await;
    let counts = handler.counts.lock().await;
    assert_eq!(counts["status"], 2);
    assert_eq!(counts["kernel_info_reply"], 1);
}

#[tokio::test]
async fn test_execute_request() {
    let (_kernel, client) = start_kernel().await;

    // send execute_request
    let handler = MessageCountHandler::new();

    let handlers = vec![Arc::new(handler.clone()) as Arc<dyn Handler>];
    let action = client.execute_request("2 + 2".to_string(), handlers).await;
    action.await;
    let counts = handler.counts.lock().await;
    // status busy -> execute_input -> stream -> status idle & execute_reply
    assert_eq!(counts["status"], 2);
    assert_eq!(counts["execute_input"], 1);
    assert_eq!(counts["execute_reply"], 1);
}

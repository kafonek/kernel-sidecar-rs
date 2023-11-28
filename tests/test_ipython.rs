use std::collections::HashMap;

use std::sync::Arc;

use kernel_sidecar_rs::handlers::{Handler, MessageCountHandler};
use kernel_sidecar_rs::utils::JupyterKernel;
use kernel_sidecar_rs::{client::Client, handlers::DebugHandler};

async fn start_ipykernel() -> (JupyterKernel, Client) {
    // Start Kernel, wait for connection file to be written, and wait for ZMQ channels to come up
    let kernel = JupyterKernel::ipython(true); // true / false is for silencing stdout
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
    let (_kernel, client) = start_ipykernel().await;

    // send kernel_info_request
    let handler = MessageCountHandler::new();
    let handlers = vec![Arc::new(handler.clone()) as Arc<dyn Handler>];
    let action = client.kernel_info_request(handlers).await;
    action.await;
    let counts = handler.counts.lock().await;
    let mut expected = HashMap::new();
    expected.insert("kernel_info_reply".to_string(), 1);
    expected.insert("status".to_string(), 2);
    assert_eq!(*counts, expected);
}

#[tokio::test]
async fn test_execute_request() {
    let (_kernel, client) = start_ipykernel().await;

    // send execute_request
    let handler = MessageCountHandler::new();

    let handlers = vec![Arc::new(handler.clone()) as Arc<dyn Handler>];
    let action = client
        .execute_request("print('hello')".to_string(), handlers)
        .await;
    action.await;
    let counts = handler.counts.lock().await;
    let mut expected = HashMap::new();
    // status busy -> execute_input -> stream -> status idle & execute_reply
    expected.insert("status".to_string(), 2);
    expected.insert("execute_input".to_string(), 1);
    expected.insert("stream".to_string(), 1);
    expected.insert("execute_reply".to_string(), 1);
    assert_eq!(*counts, expected);
}
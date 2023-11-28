use std::collections::HashMap;

use std::sync::Arc;

use kernel_sidecar_rs::client::Client;
use kernel_sidecar_rs::handlers::{Handler, MessageCountHandler};
use kernel_sidecar_rs::utils::JupyterKernel;

async fn start_evcxr() -> (JupyterKernel, Client) {
    // Start Kernel, wait for connection file to be written, and wait for ZMQ channels to come up
    let kernel = JupyterKernel::evcxr(true); // true / false is for silencing stdout
    let client = Client::new(kernel.connection_info.clone()).await;
    client.heartbeat().await;
    (kernel, client)
}

#[tokio::test]
async fn test_kernel_info() {
    let (_kernel, client) = start_evcxr().await;

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
    let (_kernel, client) = start_evcxr().await;

    // send execute_request
    let handler = MessageCountHandler::new();
    let handlers = vec![Arc::new(handler.clone()) as Arc<dyn Handler>];
    let action = client
        .execute_request("println!(\"hello world\")".to_string(), handlers)
        .await;
    action.await;
    let counts = handler.counts.lock().await;
    let mut expected = HashMap::new();
    // status busy -> execute_input -> stream -> status idle & execute_reply
    // evcxr throws in an extra execute_result that ipython doesn't with a print statement
    expected.insert("status".to_string(), 2);
    expected.insert("execute_input".to_string(), 1);
    expected.insert("stream".to_string(), 1);
    expected.insert("execute_result".to_string(), 1);
    expected.insert("execute_reply".to_string(), 1);
    assert_eq!(*counts, expected);
}

use std::sync::Arc;

use kernel_sidecar_rs::handlers::{Handler, MessageCountHandler};

mod test_utils;
use test_utils::start_kernel;

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
    // All kernel types should give status busy -> status idle -> execute reply
    assert_eq!(counts["status"], 2);
    assert_eq!(counts["execute_reply"], 1);
    // Python, Rust, and Deno will give execute_result on 2 + 2. R will give display_data.
    #[cfg(any(
        feature = "test_ipython",
        feature = "test_evcxr",
        feature = "test_deno"
    ))]
    assert_eq!(counts["execute_result"], 1);
    #[cfg(feature = "test_irkernel")]
    assert_eq!(counts["display_data"], 1);
}

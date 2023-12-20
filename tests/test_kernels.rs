use std::sync::Arc;

use kernel_sidecar_rs::client::Client;
use kernel_sidecar_rs::handlers::{Handler, MessageCountHandler};
use kernel_sidecar_rs::kernels::JupyterKernel;

// Start Kernel (type based on feature flags) and wait for ZMQ channels to come up
async fn start_kernel() -> (JupyterKernel, Client) {
    let silent = true;
    let kernel = if cfg!(feature = "test_ipython") {
        JupyterKernel::ipython(silent)
    } else if cfg!(feature = "test_evcxr") {
        JupyterKernel::evcxr(silent)
    } else if cfg!(feature = "test_irkernel") {
        JupyterKernel::irkernel(silent)
    } else if cfg!(feature = "test_deno") {
        JupyterKernel::deno(silent)
    } else {
        panic!("For tests, choose one feature flag from: test_ipython, test_evcxr, test_irkernel, test_deno")
    };
    let client = Client::new(kernel.connection_info.clone()).await;
    client.heartbeat().await;
    // Anecdotally, have noticed tests fail becaues Status messages aren't showing up as expected.
    // Theory is that heartbeat is returning but iopub isn't pushing out messages even though
    // shell is connected and accepting request / replies?
    // Could be totally wrong.
    // Separately, there may be an edge case where multiple JupyterKernel::ipython calls end up
    // with the same ports and it all blows up. TODO: fix that.
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
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

#[tokio::test]
#[cfg(feature = "test_ipython")]
async fn test_clear_output() {
    use indoc::indoc;
    let (_kernel, client) = start_kernel().await;

    // send execute_request
    let handler = MessageCountHandler::new();

    let handlers = vec![Arc::new(handler.clone()) as Arc<dyn Handler>];
    let code = indoc! {r#"
    from IPython.display import clear_output
    
    print("Hello, world!")
    clear_output()
    "#}
    .trim();
    let action = client.execute_request(code.to_string(), handlers).await;
    action.await;
    let counts = handler.counts.lock().await;
    assert_eq!(counts.get("clear_output"), Some(&1));
}

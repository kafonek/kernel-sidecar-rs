use kernel_sidecar_rs::client::Client;
use kernel_sidecar_rs::kernels::JupyterKernel;

// Start Kernel (type based on feature flags) and wait for ZMQ channels to come up
pub async fn start_kernel() -> (JupyterKernel, Client) {
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

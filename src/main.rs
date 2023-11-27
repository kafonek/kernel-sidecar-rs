use kernel_sidecar_rs::actions::Handler;
use kernel_sidecar_rs::client::{Client, ConnectionInfo};
use kernel_sidecar_rs::jupyter::response::Response;
use std::{fmt::Debug, sync::Arc};

#[derive(Debug)]
struct DebugHandler;

impl DebugHandler {
    fn new() -> Self {
        DebugHandler {}
    }
}

#[async_trait::async_trait]
impl Handler for DebugHandler {
    async fn handle(&self, msg: &Response) {
        dbg!(msg);
    }
}

#[tokio::main]
async fn main() {
    // Simple example of running a Sidecar so that it connects to a running kernel,
    // sends a kernel_info_request, and awaits until it sees the expected kernel_info_reply
    // in addition to Kernel status going to Idle. Should print out the full parsed ZMQ messages
    // coming back over iopub and shell channels.
    let connection_info = ConnectionInfo::from_file("/tmp/kernel.json")
        .expect("Make sure to run python -m ipykernel_launcher -f /tmp/kernel.json");
    let client = Client::new(connection_info).await;
    client.heartbeat().await;

    let handler = DebugHandler::new();
    let handlers = vec![Arc::new(handler) as Arc<dyn Handler>];
    // let action = client.kernel_info_request(handlers).await;
    let action = client.execute_request("2 + 2".to_owned(), handlers).await;
    action.await;
}

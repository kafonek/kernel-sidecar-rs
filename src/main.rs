use kernel_sidecar_rs::{actions::Handler, client::Client, jupyter::Response};
use std::{error::Error, fmt::Debug};
use tokio::signal;
use tokio::time::{sleep, Duration};

#[derive(Debug)]
struct DebugHandler;

#[async_trait::async_trait]
impl Handler for DebugHandler {
    async fn handle(&self, msg: &Response) {
        dbg!(msg);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // read in file from /tmp/kernel.json
    let connection_info = kernel_sidecar_rs::client::ConnectionInfo::from_file("/tmp/kernel.json")?;
    let client = Client::new(connection_info).await;

    // wait .1 seconds for zmq channels to connect
    sleep(Duration::from_millis(100)).await;

    // send kernel_info_request
    let handler = DebugHandler {};
    let handlers = vec![Box::new(handler) as Box<dyn Handler>];
    client.kernel_info_request(handlers).await;

    //signal::ctrl_c().await.expect("Failed to listen for Ctrl-C");
    sleep(Duration::from_millis(100)).await;

    Ok(())
}

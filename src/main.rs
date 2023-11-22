use kernel_sidecar_rs::actions::Handler;
use kernel_sidecar_rs::client::Client;
use kernel_sidecar_rs::jupyter::response::Response;
use std::error::Error;
use std::fmt::Debug;

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

    // send kernel_info_request
    let handler = DebugHandler {};
    let handlers = vec![Box::new(handler) as Box<dyn Handler>];
    let action = client.kernel_info_request(handlers).await;
    action.await;
    Ok(())
}

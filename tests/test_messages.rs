use std::collections::HashMap;
use std::process::{Child, Command};
use std::sync::Arc;

use kernel_sidecar_rs::actions::Handler;
use kernel_sidecar_rs::client::{Client, ConnectionInfo};
use kernel_sidecar_rs::jupyter::response::Response;
use tokio::sync::Mutex;

#[rstest::fixture]
fn ipykernel_process() -> Child {
    let cmd = Command::new("python")
        .args([
            "-m",
            "ipykernel_launcher",
            "-f",
            "/tmp/kernel_sidecar_rs_test.json",
        ])
        .spawn()
        .expect("Failed to start ipykernel");
    std::thread::sleep(std::time::Duration::from_millis(100));
    cmd
}

#[derive(Debug, Clone)]
struct MessageCountHandler {
    pub counts: Arc<Mutex<HashMap<String, usize>>>,
}

impl MessageCountHandler {
    fn new() -> Self {
        MessageCountHandler {
            counts: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait::async_trait]
impl Handler for MessageCountHandler {
    async fn handle(&self, msg: &Response) {
        let mut counts = self.counts.lock().await;
        match msg {
            Response::KernelInfo(_) => {
                let count = counts.get("kernel_info").unwrap_or(&0) + 1;
                counts.insert("kernel_info".to_string(), count);
            }

            _ => {}
        }
    }
}

#[rstest::rstest]
#[tokio::test]
async fn test_kernel_info(_ipykernel_process: Child) {
    let connection_info = ConnectionInfo::from_file("/tmp/kernel_sidecar_rs_test.json")
        .expect("Failed to read connection info from fixture");
    let client = Client::new(connection_info).await;

    // send kernel_info_request
    let handler = MessageCountHandler::new();
    let handlers = vec![Arc::new(handler.clone()) as Arc<dyn Handler>];
    let action = client.kernel_info_request(handlers).await;
    action.await;
    let counts = handler.counts.lock().await;
    assert_eq!(counts.get("kernel_info"), Some(&1));
}

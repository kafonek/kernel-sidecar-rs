use std::collections::HashMap;
use std::env;
use std::process::{Child, Command};
use std::sync::Arc;

use kernel_sidecar_rs::actions::Handler;
use kernel_sidecar_rs::client::{Client, ConnectionInfo};
use kernel_sidecar_rs::jupyter::response::Response;
use tokio::sync::Mutex;

#[rstest::fixture]
fn ipykernel_process() -> Option<Child> {
    if env::var("CI").is_ok() {
        // In CI environment, don't spawn the process
        None
    } else {
        // Spawn the ipykernel process
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
        Some(cmd)
    }
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
        let msg_type = msg.msg_type();
        let count = counts.entry(msg_type).or_insert(0);
        *count += 1;
    }
}

#[rstest::rstest]
#[serial_test::serial]
#[tokio::test]
async fn test_kernel_info(_ipykernel_process: Option<Child>) {
    let connection_info = ConnectionInfo::from_file("/tmp/kernel_sidecar_rs_test.json")
        .expect("Failed to read connection info from fixture");
    let client = Client::new(connection_info).await;

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

#[rstest::rstest]
#[serial_test::serial]
#[tokio::test]
async fn test_execute_request(_ipykernel_process: Option<Child>) {
    let connection_info = ConnectionInfo::from_file("/tmp/kernel_sidecar_rs_test.json")
        .expect("Failed to read connection info from fixture");
    let client = Client::new(connection_info).await;

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

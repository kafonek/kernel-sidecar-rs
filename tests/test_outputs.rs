#![cfg(feature = "test_ipython")]
use indoc::indoc;
use kernel_sidecar_rs::handlers::{Handler, OutputHandler};
use kernel_sidecar_rs::jupyter::iopub_content::stream::StreamName;
use kernel_sidecar_rs::notebook::Output;
use std::sync::Arc;
use tokio::sync::RwLock;

mod test_utils;
use test_utils::start_kernel;

#[derive(Debug, Clone)]
struct SimpleOutputHandler {
    pub output: Arc<RwLock<Vec<Output>>>,
}

impl SimpleOutputHandler {
    pub fn new() -> Self {
        Self {
            output: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

#[async_trait::async_trait]
impl OutputHandler for SimpleOutputHandler {
    async fn add_cell_content(&self, content: Output) {
        self.output.write().await.push(content);
        println!("add_cell_content");
    }

    async fn clear_cell_content(&self) {
        self.output.write().await.clear();
        println!("clear_cell_content");
    }
}

#[tokio::test]
async fn test_mixed_outputs() {
    // Show that stream and execute result can be mixed
    let (_kernel, client) = start_kernel().await;

    // send execute_request
    let handler = SimpleOutputHandler::new();

    let handlers = vec![Arc::new(handler.clone()) as Arc<dyn Handler>];
    let code = indoc! {r#"
    print("foo")
    print("bar")
    2 + 2
    "#}
    .trim();
    let action = client.execute_request(code.to_string(), handlers).await;
    action.await;
    let final_output = handler.output.read().await;
    assert_eq!(final_output.len(), 2);
    let stream_output = &final_output[0].as_stream().unwrap();
    assert_eq!(stream_output.name, StreamName::Stdout);
    assert_eq!(stream_output.text, "foo\nbar\n");
    let execute_result = &final_output[1].as_execute_result().unwrap();
    assert_eq!(execute_result.data["text/plain"], "4");
}

#[tokio::test]
async fn test_error_output() {
    let (_kernel, client) = start_kernel().await;

    // send execute_request
    let handler = SimpleOutputHandler::new();

    let handlers = vec![Arc::new(handler.clone()) as Arc<dyn Handler>];
    let code = indoc! {r#"
    1 / 0
    "#}
    .trim();
    let action = client.execute_request(code.to_string(), handlers).await;
    action.await;
    let final_output = handler.output.read().await;
    assert_eq!(final_output.len(), 1);
    let error_output = &final_output[0].as_error().unwrap();
    assert_eq!(error_output.ename, "ZeroDivisionError");
    assert_eq!(error_output.evalue, "division by zero");
}

#[tokio::test]
async fn test_display_data() {
    let (_kernel, client) = start_kernel().await;

    // send execute_request
    let handler = SimpleOutputHandler::new();

    let handlers = vec![Arc::new(handler.clone()) as Arc<dyn Handler>];
    let code = indoc! {r#"
    from IPython.display import display
    
    display("foo")
    "#}
    .trim();
    let action = client.execute_request(code.to_string(), handlers).await;
    action.await;
    let final_output = handler.output.read().await;
    assert_eq!(final_output.len(), 1);
    let display_data = &final_output[0].as_display_data().unwrap();
    assert_eq!(display_data.data["text/plain"], "'foo'");
}

#[tokio::test]
async fn test_clear_output() {
    let (_kernel, client) = start_kernel().await;

    // send execute_request
    let handler = SimpleOutputHandler::new();

    let handlers = vec![Arc::new(handler.clone()) as Arc<dyn Handler>];
    let code = indoc! {r#"
    from IPython.display import clear_output
    
    print("Before Clear Output")
    clear_output()
    print("After Clear Output")
    "#}
    .trim();
    let action = client.execute_request(code.to_string(), handlers).await;
    action.await;
    let final_output = handler.output.read().await;
    assert_eq!(final_output.len(), 1);
    let output = &final_output[0].as_stream().unwrap();
    assert_eq!(output.name, StreamName::Stdout);
    assert_eq!(output.text, "After Clear Output\n");
}

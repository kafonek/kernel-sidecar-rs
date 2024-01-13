#![cfg(feature = "test_ipython")]
use indoc::indoc;
use kernel_sidecar::handlers::{Handler, SimpleOutputHandler};
use kernel_sidecar::jupyter::iopub_content::stream::StreamName;
use tokio::sync::Mutex;

use std::sync::Arc;

mod test_utils;
use test_utils::start_kernel;

#[tokio::test]
async fn test_mixed_outputs() {
    // Show that stream and execute result can be mixed
    let (_kernel, client) = start_kernel().await;

    // send execute_request
    let handler = Arc::new(Mutex::new(SimpleOutputHandler::new()));
    let handlers: Vec<Arc<Mutex<dyn Handler>>> = vec![handler.clone()];

    let code = indoc! {r#"
    print("foo")
    print("bar")
    2 + 2
    "#}
    .trim();
    let action = client.execute_request(code.to_string(), handlers).await;
    action.await;
    let final_output = &handler.lock().await.output;
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
    let handler = Arc::new(Mutex::new(SimpleOutputHandler::new()));
    let handlers: Vec<Arc<Mutex<dyn Handler>>> = vec![handler.clone()];
    let code = indoc! {r#"
    1 / 0
    "#}
    .trim();
    let action = client.execute_request(code.to_string(), handlers).await;
    action.await;
    let final_output = &handler.lock().await.output;
    assert_eq!(final_output.len(), 1);
    let error_output = &final_output[0].as_error().unwrap();
    assert_eq!(error_output.ename, "ZeroDivisionError");
    assert_eq!(error_output.evalue, "division by zero");
}

#[tokio::test]
async fn test_display_data() {
    let (_kernel, client) = start_kernel().await;

    // send execute_request
    let handler = Arc::new(Mutex::new(SimpleOutputHandler::new()));
    let handlers: Vec<Arc<Mutex<dyn Handler>>> = vec![handler.clone()];
    let code = indoc! {r#"
    from IPython.display import display
    
    display("foo")
    "#}
    .trim();
    let action = client.execute_request(code.to_string(), handlers).await;
    action.await;
    let final_output = &handler.lock().await.output;
    assert_eq!(final_output.len(), 1);
    let display_data = &final_output[0].as_display_data().unwrap();
    assert_eq!(display_data.data["text/plain"], "'foo'");
}

#[tokio::test]
async fn test_clear_output() {
    let (_kernel, client) = start_kernel().await;
    let setup_action = client
        .execute_request(
            "from IPython.display import clear_output".to_string(),
            vec![],
        )
        .await;
    setup_action.await;

    let source1 = "print('foo'); clear_output()".to_string();
    let source2 = "print('bar'); clear_output(wait=True)".to_string();
    let source3 = "print('baz'); clear_output(wait=True); print('qux')".to_string();

    let handler1 = Arc::new(Mutex::new(SimpleOutputHandler::new()));
    let handler2 = Arc::new(Mutex::new(SimpleOutputHandler::new()));
    let handler3 = Arc::new(Mutex::new(SimpleOutputHandler::new()));

    let handlers1: Vec<Arc<Mutex<dyn Handler>>> = vec![handler1.clone()];
    let handlers2: Vec<Arc<Mutex<dyn Handler>>> = vec![handler2.clone()];
    let handlers3: Vec<Arc<Mutex<dyn Handler>>> = vec![handler3.clone()];

    let action1 = client.execute_request(source1, handlers1).await;
    let action2 = client.execute_request(source2, handlers2).await;
    let action3 = client.execute_request(source3, handlers3).await;
    tokio::join!(action1, action2, action3);

    assert_eq!(handler1.lock().await.output.len(), 0);

    assert_eq!(handler2.lock().await.output.len(), 1);
    assert_eq!(
        handler2.lock().await.output[0].as_stream().unwrap().text,
        "bar\n"
    );

    assert_eq!(handler3.lock().await.output.len(), 1);
    assert_eq!(
        handler3.lock().await.output[0].as_stream().unwrap().text,
        "qux\n"
    );
}

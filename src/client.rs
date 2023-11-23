use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::net::IpAddr;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::{mpsc, Notify, RwLock};
use zeromq::{DealerSocket, Socket, SocketRecv, SocketSend, SubSocket, ZmqMessage};

use crate::actions::{Action, Handler};
use crate::jupyter::message_content::kernel_info::KernelInfoRequest;
use crate::jupyter::request::Request;
use crate::jupyter::response::Response;
use crate::jupyter::wire_protocol::WireProtocol;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConnectionInfo {
    ip: IpAddr,
    transport: String,
    iopub_port: u16,
    shell_port: u16,
    hb_port: u16,
    key: String, // hmac signing key
}

impl ConnectionInfo {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let file_contents = fs::read_to_string(path)?;
        let connection_info: Self = serde_json::from_str(&file_contents)?;
        Ok(connection_info)
    }

    pub fn iopub_address(&self) -> String {
        format!("{}://{}:{}", self.transport, self.ip, self.iopub_port)
    }

    pub fn shell_address(&self) -> String {
        format!("{}://{}:{}", self.transport, self.ip, self.shell_port)
    }

    pub fn heartbeat_address(&self) -> String {
        format!("{}://{}:{}", self.transport, self.ip, self.hb_port)
    }
}

#[derive(Debug, Clone)]
pub struct Client {
    actions: Arc<RwLock<HashMap<String, mpsc::Sender<Response>>>>,
    connection_info: ConnectionInfo,
    shell_tx: mpsc::Sender<ZmqMessage>,
    shutdown_signal: Arc<Notify>,
}

impl Client {
    pub async fn new(connection_info: ConnectionInfo) -> Self {
        let actions = Arc::new(RwLock::new(HashMap::new()));
        // message passing for methods to send requests out over shell channel via shell_worker
        let (shell_tx, shell_rx) = mpsc::channel(100);

        // message passing for iopub and shell listeners into process_message_worker
        let (process_msg_tx, process_msg_rx) = mpsc::channel(100);

        // For shutting down ZMQ listeners when Client is dropped
        let shutdown_signal = Arc::new(Notify::new());

        // spawn iopub and shell listeners
        let iopub_address = connection_info.iopub_address();
        let shell_address = connection_info.shell_address();

        tokio::spawn(iopub_worker(
            iopub_address,
            process_msg_tx.clone(),
            shutdown_signal.clone(),
        ));
        tokio::spawn(shell_worker(
            shell_address,
            shell_rx,
            process_msg_tx.clone(),
            shutdown_signal.clone(),
        ));

        // spawn process_message_worker
        tokio::spawn(process_message_worker(
            process_msg_rx,
            actions.clone(),
            shutdown_signal.clone(),
        ));

        Client {
            actions,
            connection_info,
            shell_tx,
            shutdown_signal,
        }
    }

    async fn send_request(&self, request: Request, handlers: Vec<Arc<dyn Handler>>) -> Action {
        let (msg_tx, msg_rx) = mpsc::channel(100);
        let action = Action::new(request, handlers, msg_rx);
        let msg_id = action.request.msg_id();
        self.actions.write().await.insert(msg_id.clone(), msg_tx);
        let wp: WireProtocol = action.request.into_wire_protocol(&self.connection_info.key);
        let zmq_msg: ZmqMessage = wp.into();
        self.shell_tx.send(zmq_msg).await.unwrap();
        action
    }

    pub async fn kernel_info_request(&self, handlers: Vec<Arc<dyn Handler>>) -> Action {
        let request = KernelInfoRequest::new();

        self.send_request(request.into(), handlers).await
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        self.shutdown_signal.notify_waiters();
    }
}

/// The tasks listening on iopub and shell channels will push any messages they receive into this
/// receiver function. It's job is to deserialize ZmqMessage into the appropriate Jupyter message
/// and then delegate it to the appropriate Action to be handled based on parent msg_id.
async fn process_message_worker(
    mut msg_rx: mpsc::Receiver<ZmqMessage>,
    actions: Arc<RwLock<HashMap<String, mpsc::Sender<Response>>>>,
    shutdown_signal: Arc<Notify>,
) {
    loop {
        tokio::select! {
            Some(zmq_msg) = msg_rx.recv() => {
                let response: Response = zmq_msg.into();
                let msg_id = response.msg_id();
                if let Some(action) = actions.read().await.get(&msg_id) {
                    action.send(response).await.unwrap();                }
            },
            _ = shutdown_signal.notified() => {
                break;
            }
        }
    }
}

/// iopub channel background task is only responsible for listening to the iopub channel and pushing
/// messages to the process_message_worker. We never send anything out on the iopub channel.
async fn iopub_worker(
    iopub_address: String,
    msg_tx: mpsc::Sender<ZmqMessage>,
    shutdown_signal: Arc<Notify>,
) {
    let mut socket = SubSocket::new();
    socket.connect(iopub_address.as_str()).await.unwrap();
    socket.subscribe("").await.unwrap();

    loop {
        tokio::select! {
            Ok(msg) = socket.recv() => {
                msg_tx.send(msg).await.unwrap();
            },
            _ = shutdown_signal.notified() => {
                break;
            }
        }
    }
}

/// shell channel background task needs to have a way for the Client to send stuff out over shell
/// in addition to listening for replies coming back on the channel, then pushing those to the
/// process_message_worker.
async fn shell_worker(
    shell_address: String,
    mut msg_rx: mpsc::Receiver<ZmqMessage>, // Client wants to send Jupyter message over ZMQ
    msg_tx: mpsc::Sender<ZmqMessage>,       // Kernel sent a reply over ZMQ, needs to get processed
    shutdown_signal: Arc<Notify>,
) {
    let mut socket = DealerSocket::new();
    socket.connect(shell_address.as_str()).await.unwrap();

    loop {
        tokio::select! {
            Some(client_to_kernel_msg) = msg_rx.recv() => {
                socket.send(client_to_kernel_msg).await.unwrap();
            }
            kernel_to_client_msg = socket.recv() => {
                match kernel_to_client_msg {
                    Ok(msg) => {
                        msg_tx.send(msg).await.unwrap();
                    }
                    Err(e) => {
                        dbg!(e);
                    }
                }
            },
            _ = shutdown_signal.notified() => {
                break;
            }
        }
    }
}

use crate::plugin::GeyserPluginPostgresError;
use crate::types::rpc::{ServerRequest, ServerResponse};
use futures_util::stream::SplitSink;
use futures_util::{SinkExt, StreamExt};
use log::{error, info, warn};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{accept_async, WebSocketStream};

#[derive(Debug)]
pub struct WebsocketServer {
    shutdown: tokio::sync::oneshot::Sender<()>,
    jh: tokio::task::JoinHandle<()>,
}

pub type Subscribers = Arc<Mutex<HashMap<String, tokio::sync::broadcast::Sender<Message>>>>;

#[derive(Clone, Debug, Default)]
pub struct SubscriptionStore {
    pub slot_subscribers: Subscribers,
    pub transaction_subscribers: Subscribers,
}

impl WebsocketServer {
    pub async fn serve(addr: &str, subscriptions: SubscriptionStore) -> Self {
        info!("Starting websocket server on {}", addr);
        let (shutdown, receiver) = tokio::sync::oneshot::channel();
        let listener = TcpListener::bind(addr).await.unwrap();
        let jh = tokio::spawn(async move {
            info!("Websocket server started");
            //race between listener and shutdown signal, shutdown takes precedence
            while let Ok((stream, _)) = listener.accept().await {
                let peer_addr = stream.peer_addr().unwrap();
                tokio::spawn(accept_connection(peer_addr, stream, subscriptions.clone()));
            }
        });
        Self { shutdown, jh }
    }

    //shutdown the server
    pub async fn shutdown(self) {
        self.shutdown.send(()).unwrap();
        self.jh.await.unwrap();
    }
}

async fn accept_connection(peer: SocketAddr, stream: TcpStream, clients: SubscriptionStore) {
    let ws_stream = accept_async(stream)
        .await
        .expect("Error during websocket handshake");

    let (tx, rx) = tokio::sync::broadcast::channel(100);
    let (outgoing, mut incoming) = ws_stream.split();
    let client_id = peer.to_string();

    //creat a task to receive events from geyser service and send to client
    let recv_task = tokio::spawn(process_geyser_updates(rx, outgoing));

    while let Some(msg) = incoming.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                //parse the message to request
                let Ok(request) = serde_json::from_str::<ServerRequest>(&text) else {
                    warn!("Received invalid message: {:?}", text);
                    continue;
                };
                match request.method.as_str() {
                    "transaction_subscribe" => {
                        clients
                            .transaction_subscribers
                            .lock()
                            .unwrap()
                            .insert(client_id.clone(), tx.clone());
                    }
                    "slot_subscribe" => {
                        clients
                            .slot_subscribers
                            .lock()
                            .unwrap()
                            .insert(client_id.clone(), tx.clone());
                    }
                    _ => {
                        warn!("Received unhandled message: {:?}", request);
                        continue;
                    }
                }
            }
            _ => {
                warn!("Received unhandled message: {:?}", msg);
                break;
            }
        }
    }
    //remove from subscribers
    clients
        .transaction_subscribers
        .lock()
        .unwrap()
        .remove(&client_id);
    clients.slot_subscribers.lock().unwrap().remove(&client_id);

    recv_task.abort();
    info!("{} disconnected", client_id);
}

pub async fn process_geyser_updates(
    mut rx: tokio::sync::broadcast::Receiver<Message>,
    mut outgoing: SplitSink<WebSocketStream<TcpStream>, Message>,
) {
    while let Ok(msg) = rx.recv().await {
        info!("Sending message: {:?}", msg);
        if let Err(e) = outgoing.send(msg).await {
            error!("Error sending message: {:?}", e);
            break;
        }
    }
}

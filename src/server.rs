use crate::plugin::GeyserPluginPostgresError;
use crate::types::channel_message::ChannelMessage;
use crate::types::filters::{Filters, SubscriptionType};
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

#[derive(Clone, Debug)]
pub struct ServerConfig {
    pub slot_subscribers: tokio::sync::broadcast::Sender<ChannelMessage>,
    pub transaction_subscribers: tokio::sync::broadcast::Sender<ChannelMessage>,
}

impl WebsocketServer {
    pub async fn serve(addr: &str, subscriptions: ServerConfig) -> Self {
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

async fn accept_connection(peer: SocketAddr, stream: TcpStream, config: ServerConfig) {
    let ws_stream = accept_async(stream)
        .await
        .expect("Error during websocket handshake");

    let (outgoing, mut incoming) = ws_stream.split();
    let client_id = peer.to_string();

    let (filers_tx, filters_rx) = tokio::sync::mpsc::channel(1);
    //creat a task to receive events from geyser service and send to client
    let recv_task = tokio::spawn(send_geyser_updates(config, filters_rx, outgoing));

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
                        filers_tx
                            .send(Filters {
                                is_vote: false,
                                include_accounts: vec![],
                                subscription_type: SubscriptionType::Transaction,
                            })
                            .await
                            .unwrap();
                    }
                    "slot_subscribe" => {
                        filers_tx
                            .send(Filters {
                                is_vote: false,
                                include_accounts: vec![],
                                subscription_type: SubscriptionType::Slot,
                            })
                            .await
                            .unwrap();
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
    recv_task.abort();
    info!("{} disconnected", client_id);
}

pub async fn send_geyser_updates(
    config: ServerConfig,
    mut filters_rx: tokio::sync::mpsc::Receiver<Filters>,
    mut outgoing: SplitSink<WebSocketStream<TcpStream>, Message>,
) {
    let filter = filters_rx.recv().await;
    if let Some(filter) = filter {
        let mut rx = match filter.subscription_type {
            SubscriptionType::Slot => config.slot_subscribers.subscribe(),
            SubscriptionType::Transaction => config.transaction_subscribers.subscribe(),
            _ => {
                return;
            }
        };

        while let Ok(msg) = rx.recv().await {
            let response = ServerResponse {
                result: msg.try_into().unwrap(),
            };
            if let Err(e) = outgoing
                .send(Message::Text(serde_json::to_string(&response).unwrap()))
                .await
            {
                error!("Error sending message: {:?}", e);
                break;
            }
        }
    }
}

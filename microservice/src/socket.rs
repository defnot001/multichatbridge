use axum::extract::ws::{Message, WebSocket};
use futures::{sink::SinkExt, stream::StreamExt};
use futures_util::stream::{SplitSink, SplitStream};

use std::net::SocketAddr;
use std::ops::ControlFlow;

use crate::message::process_message;

/// Websocket statemachine, one will be spawned per connection
pub async fn handle_socket(socket: WebSocket, address: SocketAddr) {
    let mut handler = WebSocketHandler::new(socket, address);

    // ping the client, if it does not respond, drop the connection
    if handler.ping_client().await.is_break() {
        return;
    }

    // wait for a single message from the client (usually the pong), if it does not respond, drop the connection
    if handler.receive_single_message().await.is_break() {
        return;
    }

    tokio::spawn(async move {
        while let Some(Ok(msg)) = handler.receiver.next().await {
            if process_message(msg.clone(), address).is_break() {
                break;
            }

            if handler.sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    println!("Websocket context {address} destroyed");
}

/// This struct holds the `websocket connection` and the `client's address` and provides
/// convenience methods for sending and receiving messages.
#[derive(Debug)]
struct WebSocketHandler {
    address: SocketAddr,
    sender: SplitSink<WebSocket, Message>,
    receiver: SplitStream<WebSocket>,
}

impl WebSocketHandler {
    /// Creates a new `SplitWebsocketHandler` from a `WebSocket` and a `SocketAddr`.
    fn new(socket: WebSocket, address: SocketAddr) -> Self {
        let (sender, receiver) = socket.split();

        Self {
            address,
            sender,
            receiver,
        }
    }

    async fn ping_client(&mut self) -> ControlFlow<(), ()> {
        if self.sender.send(Message::Ping(vec![1, 2, 3])).await.is_ok() {
            println!("Pinged {} successfully", self.address);

            ControlFlow::Continue(())
        } else {
            println!(
                "Could not send ping to {}, closing connection...",
                self.address
            );

            ControlFlow::Break(())
        }
    }

    /// Waits for a single message from the client and returns `ControlFlow::Continue` if the
    /// message was received successfully or `ControlFlow::Break` if the message could not be
    /// received if there was an error or if the client closed the connection. Usually this would
    /// be used after sending a ping to the client to check if the client is still alive. This will
    /// block the current task, but will not block other client's connections.
    async fn receive_single_message(&mut self) -> ControlFlow<(), ()> {
        if let Some(msg) = self.receiver.next().await {
            if let Ok(msg) = msg {
                if process_message(msg, self.address).is_break() {
                    return ControlFlow::Break(());
                }

                ControlFlow::Continue(())
            } else {
                println!("client {} abruptly disconnected", self.address);
                ControlFlow::Break(())
            }
        } else {
            println!("stream at {} has closed", self.address);
            ControlFlow::Break(())
        }
    }
}

use std::net::SocketAddr;

use axum::extract::ws::WebSocket;
use futures_util::StreamExt;

use crate::ctx::ctx_client::ClientCtx;
use crate::message::process_message;
use crate::{ActiveConnection, ActiveConnections};

pub async fn handle_socket(
    socket: WebSocket,
    address: SocketAddr,
    subscriptions: Vec<String>,
    client_ctx: ClientCtx,
    conn: ActiveConnection,
    active_connections: ActiveConnections,
) {
    let receive_task = tokio::spawn(async move {
        let (mut ws_sender, mut ws_receiver) = socket.split();
        while let Some(Ok(msg)) = ws_receiver.next().await {
            if process_message(msg.clone(), address).is_break() {
                break;
            }
        }
    });
}

// #[derive(Debug)]
// struct WebSocketHandler {
//     socket: WebSocket,
//     identifier: Identifier,
//     subscriptions: Vec<String>,
//     address: SocketAddr,
//     connection: ActiveConnection,
//     active_connections: ActiveConnections,
// }
//
// impl WebSocketHandler {
//     fn new(
//         socket: WebSocket,
//         identifier: Identifier,
//         subscriptions: Vec<String>,
//         address: SocketAddr,
//         connection: ActiveConnection,
//         active_connections: ActiveConnections,
//     ) -> Self {
//         Self {
//             socket,
//             identifier,
//             subscriptions,
//             address,
//             connection,
//             active_connections,
//         }
//     }
//
//     async fn check_health(&mut self) {
//         let mut interval = interval(Duration::from_secs(60));
//
//         loop {
//             interval.tick().await;
//
//             if !self.check_connection_alive().await {
//                 self.disconnect_client().await;
//                 break;
//             }
//         }
//     }
//
//     async fn check_connection_alive(&mut self) -> bool {
//         let ping_message = Message::Ping(vec![1, 2, 3]);
//
//         match tokio::time::timeout(Duration::from_secs(5), self.socket.send(ping_message)).await {
//             Ok(Ok(())) => {
//                 match tokio::time::timeout(Duration::from_secs(5), self.receiver.next()).await {
//                     Ok(Some(Ok(Message::Pong(_)))) => true,
//                     _ => false,
//                 }
//             }
//             _ => {
//                 // Error occurred while sending ping message, connection is considered inactive
//                 false
//             }
//         }
//     }
//
//     async fn disconnect_client(&mut self) {
//         if let Err(e) = self.sender.close().await {
//             warn!(
//                 "Failed to close connection for {} at {}: {}",
//                 self.identifier.to_string(),
//                 self.address,
//                 e
//             )
//         } else {
//             info!(
//                 "Closed connection to {} at {}",
//                 self.identifier.to_string(),
//                 self.address
//             )
//         }
//     }
// }

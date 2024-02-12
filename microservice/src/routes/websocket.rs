use std::net::SocketAddr;

use axum::routing::post;
use axum::{
    extract::{ConnectInfo, WebSocketUpgrade},
    response::IntoResponse,
    Router,
};
use sqlx::SqlitePool;
use tracing::info;

use crate::middleware::mw_auth_websocket::mw_websocket_auth;
use crate::websocket::websocket_handler::handle_socket;
pub fn websocket_routes(db_pool: SqlitePool) -> Router {
    Router::new()
        .route("/ws", post(handle_websocket))
        .route_layer(axum::middleware::from_fn_with_state(
            db_pool.clone(),
            mw_websocket_auth,
        ))
        .with_state(db_pool)
}

pub async fn handle_websocket(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    info!("User agent at {addr} connected.");

    ws.on_upgrade(move |socket| handle_socket(socket, addr))
}

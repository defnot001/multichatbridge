use std::net::SocketAddr;

use axum::extract::State;
use axum::routing::get;
use axum::{
    extract::{ConnectInfo, WebSocketUpgrade},
    response::IntoResponse,
    Extension, Router,
};
use sqlx::SqlitePool;
use tracing::info;

use crate::config::Config;
use crate::ctx::ctx_client::ClientCtx;
use crate::middleware::mw_auth_websocket::mw_websocket_auth;
use crate::websocket::websocket_handler::handle_socket;
use crate::{ActiveConnection, ActiveConnections, AppState};

pub fn websocket_routes(app_state: AppState) -> Router {
    Router::new()
        .route("/ws", get(handle_websocket))
        .route_layer(axum::middleware::from_fn_with_state(
            app_state.clone(),
            mw_websocket_auth,
        ))
        .with_state(app_state)
}

pub async fn handle_websocket(
    Extension(client_ctx): Extension<ClientCtx>,
    Extension(subscriptions): Extension<Vec<String>>,
    state: State<AppState>,
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    info!(
        "User agent {} connected with subscriptions: {}",
        client_ctx.identifier.to_string(),
        subscriptions.join(", ")
    );

    let conn = ActiveConnection::new(client_ctx.identifier.clone(), subscriptions.clone());
    let active_conn = state.active_connections.clone();

    active_conn.clone().lock().await.push(conn.clone());

    println!(
        "Active Connections after pushing: {:#?}",
        &state.active_connections
    );

    ws.on_upgrade(move |socket| {
        handle_socket(socket, addr, subscriptions, client_ctx, conn, active_conn)
    })
}

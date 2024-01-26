use axum::extract::{connect_info::ConnectInfo, ws::WebSocketUpgrade};
use axum::response::IntoResponse;
use axum_extra::TypedHeader;
use std::net::SocketAddr;

use crate::socket::handle_socket;

pub async fn handle_websocket(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    content_type: Option<TypedHeader<headers::ContentType>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("unknown user_agent")
    };

    let content_type = if let Some(TypedHeader(content_type)) = content_type {
        content_type.to_string()
    } else {
        String::from("Unknown content_type")
    };

    println!("{user_agent} at {addr} connected. Content type is {content_type}.");

    ws.on_upgrade(move |socket| handle_socket(socket, addr))
}

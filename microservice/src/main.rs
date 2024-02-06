// #![allow(unused_imports, dead_code, unused_variables)]
mod config;
mod database;
mod handlers;
mod message;
mod routes;

use axum::{
    routing::{get, post},
    Router,
};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    SqlitePool,
};
use std::net::{IpAddr, SocketAddr};
use tracing::info;
use tracing_subscriber::EnvFilter;

use crate::config::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .without_time()
        .with_target(false)
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let db_pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(
            SqliteConnectOptions::new()
                .create_if_missing(true)
                .filename("data.db"),
        )
        .await?;

    sqlx::migrate!("./database/migrations")
        .run(&db_pool)
        .await?;

    let app_state = AppState {
        db_pool,
        config: Config::load_from_env()?,
    };

    let app = Router::new()
        .route("/ws", get(handlers::websocket::handle_websocket))
        .route("/config", post(handlers::config::handle_config))
        .with_state(app_state.clone())
        .nest("/admin", routes::admin::admin_router(app_state.clone()))
        .fallback(handlers::not_found::handle_404);

    let listener = tokio::net::TcpListener::bind(SocketAddr::new(
        IpAddr::V4(app_state.config.SERVER_IP),
        app_state.config.SERVER_PORT,
    ))
    .await?;

    info!("Listening on {}", listener.local_addr()?);

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}

#[derive(Debug, Clone)]
struct AppState {
    db_pool: SqlitePool,
    config: Config,
}

use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, Mutex};

use axum::Router;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use tokio::sync::Mutex as TokioMutex;
use tracing::info;
use tracing_subscriber::EnvFilter;

use crate::config::Config;
use crate::ctx::ctx_client::Identifier;
use crate::routes::{admin::admin_routes, config::config_routes, websocket::websocket_routes};

mod config;
mod ctx;
mod database_utils;
mod message;
mod middleware;
mod model;
mod routes;
mod websocket;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let config = Config::load_from_env()?;

    let db_pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&config.DATABASE_URL)
        .await?;

    let app_state = AppState::new(db_pool, config);

    let app = Router::new()
        .merge(websocket_routes(app_state.db_pool.clone()))
        .nest("/config", config_routes(app_state.db_pool.clone()))
        .nest("/admin", admin_routes(app_state.clone()))
        .fallback(routes::not_found::handle_404);

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
    active_connections: Arc<TokioMutex<Vec<ActiveConnection>>>,
}

impl AppState {
    pub fn new(db_pool: SqlitePool, config: Config) -> Self {
        Self {
            db_pool,
            config,
            active_connections: Arc::new(TokioMutex::new(Vec::new())),
        }
    }
}

#[derive(Debug, Clone)]
struct ActiveConnection {
    identifier: Identifier,
    subscriptions: Vec<String>,
}

use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::post;
use axum::{
    extract::{Json, State},
    middleware, Extension, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use crate::ctx::ctx_client::ClientCtx;
use crate::middleware::mw_auth_client::mw_client_auth;
use crate::model::config::{ClientConfig, ConfigModelController};

pub fn config_routes(db_pool: SqlitePool) -> Router {
    Router::new()
        .route("/add", post(handle_config_post))
        .route_layer(middleware::from_fn_with_state(
            db_pool.clone(),
            mw_client_auth,
        ))
        .with_state(db_pool)
}

pub async fn handle_config_post(
    Extension(client_ctx): Extension<ClientCtx>,
    State(db_pool): State<SqlitePool>,
    Json(body): Json<ConfigRequestBody>,
) -> impl IntoResponse {
    match ConfigModelController::add_or_update_config(
        client_ctx.client_id.as_str(),
        &body.subscriptions,
        db_pool,
    )
    .await
    {
        Ok(client_config) => {
            tracing::info!("Client Config Added or Updated: {:#?}", client_config);

            let response_body: ConfigResponseBody = client_config.into();
            let status_code = StatusCode::CREATED;

            (status_code, Json(response_body)).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to add or update config: {}", e);

            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigRequestBody {
    pub subscriptions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigResponseBody {
    pub client_id: String,
    pub subscriptions: Vec<String>,
}

impl From<ClientConfig> for ConfigResponseBody {
    fn from(config: ClientConfig) -> Self {
        Self {
            client_id: config.client_id,
            subscriptions: config.subscriptions,
        }
    }
}

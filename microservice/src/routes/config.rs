use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{
    extract::{Extension, Json, State},
    middleware, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use crate::ctx::ctx_client::{ClientCtx, Identifier};
use crate::middleware::mw_auth_client::mw_client_auth;
use crate::model::config::{ClientConfig, ConfigModelController};
use crate::model::user::UserModelController;

pub fn config_routes(db_pool: SqlitePool) -> Router {
    Router::new()
        .route("/get", get(handle_config_get))
        .route("/list", get(handle_config_list))
        .route("/add", post(handle_config_post))
        .route_layer(middleware::from_fn_with_state(
            db_pool.clone(),
            mw_client_auth,
        ))
        .with_state(db_pool)
}

pub async fn handle_config_get(
    State(db_pool): State<SqlitePool>,
    Extension(client_ctx): Extension<ClientCtx>,
) -> impl IntoResponse {
    match ConfigModelController::get_config_by_identifier(client_ctx.identifier, db_pool).await {
        Ok(client_config) => {
            tracing::info!("Client Config Requested: {:#?}", client_config);
            Json(client_config).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to get config: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn handle_config_list(
    State(db_pool): State<SqlitePool>,
    Extension(client_ctx): Extension<ClientCtx>,
) -> impl IntoResponse {
    match ConfigModelController::get_config_by_server_id(client_ctx.identifier.server_id(), db_pool)
        .await
    {
        Ok(client_configs) => {
            tracing::info!("Client Configs Requested: {:#?}", client_configs);
            Json(client_configs).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to list configs: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn handle_config_post(
    State(db_pool): State<SqlitePool>,
    Extension(client_ctx): Extension<ClientCtx>,
    Json(body): Json<ConfigRequestBody>,
) -> impl IntoResponse {
    if let Err(e) = is_config_allowed(client_ctx.identifier(), &body, db_pool.clone()).await {
        return (
            StatusCode::FORBIDDEN,
            format!("Not allowed to add config: {}", e),
        )
            .into_response();
    }

    match ConfigModelController::add_or_update_config(
        client_ctx.identifier(),
        &body.subscriptions,
        db_pool,
    )
    .await
    {
        Ok(client_config) => {
            tracing::info!("Client Config Added or Updated: {:#?}", client_config);
            (StatusCode::CREATED, Json(client_config)).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to add or update config: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn is_config_allowed(
    identifier: &Identifier,
    body: &ConfigRequestBody,
    db_pool: SqlitePool,
) -> anyhow::Result<()> {
    let user = UserModelController::get_user_by_id(identifier.server_id(), db_pool).await?;

    for sub in &body.subscriptions {
        if !user.server_list.contains(sub) {
            let error_msg = format!(
                "Client {} is not allowed to subscribe to {}",
                identifier, sub
            );

            tracing::error!(error_msg);
            return Err(anyhow::anyhow!(error_msg));
        }
    }

    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigRequestBody {
    pub subscriptions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigResponseBody {
    pub server_id: String,
    pub subscriptions: Vec<String>,
}

impl From<ClientConfig> for ConfigResponseBody {
    fn from(config: ClientConfig) -> Self {
        Self {
            server_id: config.server_id,
            subscriptions: config.subscriptions,
        }
    }
}

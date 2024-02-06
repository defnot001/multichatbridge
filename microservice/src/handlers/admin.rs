use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_extra::TypedHeader;
use headers::{authorization::Bearer, Authorization};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{pool::PoolConnection, Sqlite, SqliteConnection, SqlitePool};

use crate::{config::Config, AppState};

pub async fn handle_admin_list() -> impl IntoResponse {
    "Admin page"
}

pub async fn handle_admin_add(
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    State(app_state): State<AppState>,
    Json(body): Json<AdminPostBody>,
) -> impl IntoResponse {
    if !check_auth(auth, app_state.config.clone()).await {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    let Ok(mut connection) = acquire_connection(&app_state.db_pool).await else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };

    if add_user(body, &mut connection).await.is_ok() {
        return Json(AdminPostResponse { success: true }).into_response();
    }

    StatusCode::INTERNAL_SERVER_ERROR.into_response()
}

pub async fn handle_admin_delete() -> impl IntoResponse {
    "Admin delete"
}

pub async fn handle_admin_update() -> impl IntoResponse {
    "Admin update"
}

async fn acquire_connection(db_pool: &SqlitePool) -> anyhow::Result<PoolConnection<Sqlite>> {
    db_pool.acquire().await.map_err(|e| {
        tracing::error!("Failed to acquire a connection: {}", e);
        anyhow::anyhow!("Failed to acquire a connection")
    })
}

async fn check_auth(auth: Authorization<Bearer>, config: Config) -> bool {
    println!("Auth: {:?} | Config: {:?}", auth, config.ADMIN_TOKEN);
    let config_admin_token = config.ADMIN_TOKEN;

    auth.0.token() == config_admin_token
}

async fn add_user(data: AdminPostBody, connection: &mut SqliteConnection) -> anyhow::Result<()> {
    let hashed_password = hash_password(&data.server_id);

    sqlx::query("INSERT INTO users (server_id, server_list, auth_token) VALUES (?, ?, ?)")
        .bind(&data.server_id)
        .bind(serde_json::to_string(&data.server_list)?)
        .bind(hashed_password)
        .execute(connection)
        .await?;

    Ok(())
}

fn hash_password(password: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password);
    hasher.update(b"lkajsdf982");
    format!("{:x}", hasher.finalize())
}

#[derive(Debug, Deserialize)]
pub struct AdminPostBody {
    pub server_id: String,
    pub server_list: Vec<String>,
    pub auth_token: String,
}

#[derive(Debug, Serialize)]
struct AdminPostResponse {
    success: bool,
}

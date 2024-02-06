use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_extra::TypedHeader;
use headers::{authorization::Bearer, Authorization};
use serde::Serialize;
use serde_json::json;
use sqlx::{pool::PoolConnection, Sqlite, SqlitePool};

use crate::{
    config::Config,
    database::{AdminDeleteBody, AdminPostBody, AdminUpdateBody, DatabaseHelper},
    AppState,
};

pub async fn handle_admin_list(
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    State(app_state): State<AppState>,
) -> impl IntoResponse {
    if !check_auth(auth, app_state.config.clone()).await {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    let Ok(mut connection) = acquire_connection(&app_state.db_pool).await else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };

    if let Ok(users) = DatabaseHelper::list_users(&mut connection).await {
        return Json(users).into_response();
    }

    StatusCode::INTERNAL_SERVER_ERROR.into_response()
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

    match DatabaseHelper::add_user(body, &mut connection).await {
        Ok(_) => {
            return Json(AdminResponseBody {
                success: true,
                reason: None,
            })
            .into_response();
        }
        Err(e) => {
            let status_code = StatusCode::CONFLICT;
            let body = Json(AdminResponseBody {
                success: false,
                reason: Some(e.to_string()),
            });

            return (status_code, body).into_response();
        }
    }
}

pub async fn handle_admin_delete(
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    State(app_state): State<AppState>,
    Json(body): Json<AdminDeleteBody>,
) -> impl IntoResponse {
    if !check_auth(auth, app_state.config.clone()).await {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    let Ok(mut connection) = acquire_connection(&app_state.db_pool).await else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };

    match DatabaseHelper::delete_user(body, &mut connection).await {
        Ok(_) => {
            return Json(AdminResponseBody {
                success: true,
                reason: None,
            })
            .into_response();
        }
        Err(e) => {
            return Json(AdminResponseBody {
                success: false,
                reason: Some(e.to_string()),
            })
            .into_response();
        }
    }
}

pub async fn handle_admin_update(
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    State(app_state): State<AppState>,
    Json(body): Json<AdminUpdateBody>,
) -> impl IntoResponse {
    if !check_auth(auth, app_state.config.clone()).await {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    let Ok(mut connection) = acquire_connection(&app_state.db_pool).await else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };

    match DatabaseHelper::update_user(body, &mut connection).await {
        Ok(user) => {
            let response = json!({
                "success": true,
                "reason": null,
                "updated_user": user,
            });

            return Json(response).into_response();
        }
        Err(e) => {
            return Json(AdminResponseBody {
                success: false,
                reason: Some(e.to_string()),
            })
            .into_response();
        }
    }
}

async fn acquire_connection(db_pool: &SqlitePool) -> anyhow::Result<PoolConnection<Sqlite>> {
    db_pool.acquire().await.map_err(|e| {
        tracing::error!("Failed to acquire a connection: {}", e);
        anyhow::anyhow!("Failed to acquire a connection")
    })
}

async fn check_auth(auth: Authorization<Bearer>, config: Config) -> bool {
    auth.0.token() == config.ADMIN_TOKEN
}

#[derive(Debug, Serialize)]
struct AdminResponseBody {
    success: bool,
    reason: Option<String>,
}

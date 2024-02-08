use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;
use serde_json::json;
use sqlx::{pool::PoolConnection, Sqlite, SqlitePool};

use crate::{
    database::{AdminDeleteBody, AdminPostBody, AdminUpdateBody, DatabaseHelper},
    AppState,
};

pub async fn handle_admin_list(State(app_state): State<AppState>) -> impl IntoResponse {
    let Ok(mut connection) = acquire_connection(&app_state.db_pool).await else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };

    if let Ok(users) = DatabaseHelper::list_users(&mut connection).await {
        tracing::info!("User List Requested: {:#?}", users);
        return Json(users).into_response();
    }

    tracing::error!("Failed to list users");
    StatusCode::INTERNAL_SERVER_ERROR.into_response()
}

pub async fn handle_admin_add(
    State(app_state): State<AppState>,
    Json(body): Json<AdminPostBody>,
) -> impl IntoResponse {
    let Ok(mut connection) = acquire_connection(&app_state.db_pool).await else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };

    match DatabaseHelper::add_user(body.clone(), &mut connection).await {
        Ok(_) => {
            tracing::info!("User Added: {:#?}", body);
            return Json(AdminResponseBody {
                success: true,
                reason: None,
            })
            .into_response();
        }
        Err(e) => {
            tracing::error!("Failed to add user: {}", e);
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
    State(app_state): State<AppState>,
    Json(body): Json<AdminDeleteBody>,
) -> impl IntoResponse {
    let Ok(mut connection) = acquire_connection(&app_state.db_pool).await else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };

    match DatabaseHelper::delete_user(body.clone(), &mut connection).await {
        Ok(_) => {
            tracing::info!("User Deleted: {:#?}", body);
            return Json(AdminResponseBody {
                success: true,
                reason: None,
            })
            .into_response();
        }
        Err(e) => {
            tracing::error!("Failed to delete user: {}", e);
            return Json(AdminResponseBody {
                success: false,
                reason: Some(e.to_string()),
            })
            .into_response();
        }
    }
}

pub async fn handle_admin_update(
    State(app_state): State<AppState>,
    Json(body): Json<AdminUpdateBody>,
) -> impl IntoResponse {
    let Ok(mut connection) = acquire_connection(&app_state.db_pool).await else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };

    match DatabaseHelper::update_user(body.clone(), &mut connection).await {
        Ok(user) => {
            tracing::info!("User Updated: {:#?}", user);
            let response = json!({
                "success": true,
                "reason": null,
                "updated_user": user,
            });

            return Json(response).into_response();
        }
        Err(e) => {
            tracing::error!("Failed to update user: {}", e);
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

#[derive(Debug, Serialize)]
struct AdminResponseBody {
    success: bool,
    reason: Option<String>,
}

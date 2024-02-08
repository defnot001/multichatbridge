use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;
use serde_json::json;

use crate::{
    database::{AdminDeleteBody, AdminPostBody, AdminUpdateBody, DatabaseHelper},
    AppState,
};

pub async fn handle_admin_list(State(app_state): State<AppState>) -> impl IntoResponse {
    if let Ok(users) = DatabaseHelper::list_users(app_state.db_pool).await {
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
    match DatabaseHelper::add_user(body.clone(), app_state.db_pool).await {
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
    match DatabaseHelper::delete_user(body.clone(), app_state.db_pool).await {
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
    match DatabaseHelper::update_user(body.clone(), app_state.db_pool).await {
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

#[derive(Debug, Serialize)]
struct AdminResponseBody {
    success: bool,
    reason: Option<String>,
}

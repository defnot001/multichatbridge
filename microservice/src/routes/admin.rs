use axum::{
    extract::State,
    http::StatusCode,
    middleware,
    response::IntoResponse,
    routing::{delete, get, patch, post},
    Json, Router,
};
use serde::Serialize;
use serde_json::json;

use crate::{
    model::user::{AdminDeleteBody, AdminPostBody, AdminUpdateBody, UserModelController},
    AppState,
};

pub fn admin_routes(app_state: AppState) -> Router {
    Router::new()
        .route("/list", get(handle_admin_list))
        .route("/add", post(handle_admin_add))
        .route("/delete", delete(handle_admin_delete))
        .route("/update", patch(handle_admin_update))
        .with_state(app_state.clone())
        .route_layer(middleware::from_fn_with_state(
            app_state,
            crate::middleware::mw_auth_admin::mw_admin_auth,
        ))
}

pub async fn handle_admin_list(State(app_state): State<AppState>) -> impl IntoResponse {
    if let Ok(users) = UserModelController::list_users(app_state.db_pool).await {
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
    match UserModelController::add_user(body.clone(), app_state.db_pool).await {
        Ok(_) => {
            tracing::info!("User Added: {:#?}", body);
            Json(AdminResponseBody {
                success: true,
                reason: None,
            })
            .into_response()
        }
        Err(e) => {
            tracing::error!("Failed to add user: {}", e);
            let status_code = StatusCode::CONFLICT;
            let body = Json(AdminResponseBody {
                success: false,
                reason: Some(e.to_string()),
            });

            (status_code, body).into_response()
        }
    }
}

pub async fn handle_admin_delete(
    State(app_state): State<AppState>,
    Json(body): Json<AdminDeleteBody>,
) -> impl IntoResponse {
    match UserModelController::delete_user(body.clone(), app_state.db_pool).await {
        Ok(_) => {
            tracing::info!("User Deleted: {:#?}", body);
            Json(AdminResponseBody {
                success: true,
                reason: None,
            })
            .into_response()
        }
        Err(e) => {
            tracing::error!("Failed to delete user: {}", e);
            Json(AdminResponseBody {
                success: false,
                reason: Some(e.to_string()),
            })
            .into_response()
        }
    }
}

pub async fn handle_admin_update(
    State(app_state): State<AppState>,
    Json(body): Json<AdminUpdateBody>,
) -> impl IntoResponse {
    match UserModelController::update_user(body.clone(), app_state.db_pool).await {
        Ok(user) => {
            tracing::info!("User Updated: {:#?}", user);
            let response = json!({
                "success": true,
                "reason": null,
                "updated_user": user,
            });

            Json(response).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to update user: {}", e);
            Json(AdminResponseBody {
                success: false,
                reason: Some(e.to_string()),
            })
            .into_response()
        }
    }
}

#[derive(Debug, Serialize)]
struct AdminResponseBody {
    success: bool,
    reason: Option<String>,
}

use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use axum_extra::TypedHeader;
use headers::{authorization::Bearer, Authorization};
use sqlx::SqlitePool;

use crate::{AppState};
use crate::model::user::UserModelController;

pub async fn mw_client_auth(
    State(app_state): State<AppState>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let client_id_header = req
        .headers()
        .get("X-Client-ID")
        .and_then(|header| header.to_str().ok());

    let Some(client_id) = client_id_header else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    if let Err(e) = authorize_client(client_id, &app_state.db_pool, &auth).await {
        if e.to_string() == "Unauthorized Admin Request" {
            return Err(StatusCode::UNAUTHORIZED);
        }

        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    Ok(next.run(req).await)
}

async fn authorize_client(
    client_id: &str,
    db_pool: &SqlitePool,
    auth_token: &Authorization<Bearer>,
) -> anyhow::Result<()> {
    let user = match UserModelController::get_user_by_id(client_id, db_pool).await {
        Ok(user) => user,
        Err(e) => return Err(e),
    };

    if user.hashed_auth_token != UserModelController::hash_password(auth_token.token().to_string())
    {
        tracing::error!("Unauthorized Admin Request Attempt");
        return Err(anyhow::anyhow!("Unauthorized Admin Request"));
    }

    Ok(())
}

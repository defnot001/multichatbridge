use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::Response;
use axum_extra::TypedHeader;
use headers::authorization::Bearer;
use headers::Authorization;
use sqlx::SqlitePool;

use crate::config::Config;
use crate::ctx::ctx_client::{ClientCtx, Identifier};
use crate::database_utils::hash_password;
use crate::model::config::ConfigModelController;
use crate::model::user::UserModelController;
use crate::AppState;

pub async fn mw_websocket_auth(
    app_state: State<AppState>,
    client_ctx: ClientCtx,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    check_auth_token(
        client_ctx.identifier(),
        &auth,
        &app_state.db_pool,
        &app_state.config.SALT,
    )
    .await?;
    let subscriptions =
        get_subscriptions(client_ctx.identifier.clone(), &app_state.db_pool).await?;

    req.extensions_mut().insert(client_ctx.clone());
    req.extensions_mut().insert(subscriptions);

    tracing::info!("Wesocket Middleware Authenticated Client: {:?}", client_ctx);

    Ok(next.run(req).await)
}

async fn check_auth_token(
    identifier: &Identifier,
    auth_token: &Authorization<Bearer>,
    db_pool: &SqlitePool,
    config_salt: &str,
) -> Result<(), StatusCode> {
    let Ok(user) = UserModelController::get_user_by_id(identifier.server_id(), db_pool).await
    else {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    if user.hashed_auth_token != hash_password(auth_token.token().to_string(), config_salt) {
        tracing::error!("Unauthorized Websocket Request Attempt");
        return Err(StatusCode::UNAUTHORIZED);
    }

    Ok(())
}

async fn get_subscriptions(
    identifier: Identifier,
    db_pool: &SqlitePool,
) -> Result<Vec<String>, StatusCode> {
    match ConfigModelController::get_config_by_identifier(&identifier, db_pool).await {
        Ok(config) => Ok(config.subscriptions),
        Err(e) => {
            if e.to_string() == "No Config Found" {
                Err(StatusCode::NOT_FOUND)
            } else {
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use axum_extra::TypedHeader;
use headers::{authorization::Bearer, Authorization};
use sqlx::SqlitePool;

use crate::ctx::ctx_client::{ClientCtx, Identifier};
use crate::database_utils::DatabaseUtils;
use crate::model::user::UserModelController;

pub async fn mw_client_auth(
    client_ctx: ClientCtx,
    State(db_pool): State<SqlitePool>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if let Err(e) = authorize_client(client_ctx.identifier(), db_pool, &auth).await {
        if e.to_string() == "Unauthorized Admin Request" {
            return Err(StatusCode::UNAUTHORIZED);
        }

        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    req.extensions_mut().insert(client_ctx);

    Ok(next.run(req).await)
}

async fn authorize_client(
    identifier: &Identifier,
    db_pool: SqlitePool,
    auth_token: &Authorization<Bearer>,
) -> anyhow::Result<()> {
    let user = match UserModelController::get_user_by_id(identifier.server_id(), db_pool).await {
        Ok(user) => user,
        Err(e) => return Err(e),
    };

    if user.hashed_auth_token != DatabaseUtils::hash_password(auth_token.token().to_string()) {
        tracing::error!("Unauthorized Admin Request Attempt");
        return Err(anyhow::anyhow!("Unauthorized Admin Request"));
    }

    Ok(())
}

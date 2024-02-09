use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use axum_extra::TypedHeader;
use headers::{authorization::Bearer, Authorization};

use crate::{config::Config, AppState};

pub async fn mw_admin_auth(
    State(app_state): State<AppState>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if authorize_user(&app_state.config, &auth).is_ok() {
        return Ok(next.run(req).await);
    }

    Err(StatusCode::UNAUTHORIZED)
}

fn authorize_user(config: &Config, auth_token: &Authorization<Bearer>) -> anyhow::Result<()> {
    if auth_token.0.token() != config.ADMIN_TOKEN {
        tracing::error!("Unauthorized Admin Request Attempt");
        return Err(anyhow::anyhow!("Unauthorized Admin Request"));
    }

    Ok(())
}

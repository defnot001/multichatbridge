use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use axum_extra::TypedHeader;
use headers::{authorization::Bearer, Authorization};

use crate::{config::Config, AppState};

struct AdminUser {
    authorized: bool,
}

pub async fn admin_auth(
    State(app_state): State<AppState>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if let Some(user) = authorize_user(&app_state.config, &auth).await {
        if user.authorized {
            return Ok(next.run(req).await);
        }
    }

    Err(StatusCode::UNAUTHORIZED)
}

async fn authorize_user(config: &Config, auth_token: &Authorization<Bearer>) -> Option<AdminUser> {
    if auth_token.0.token() != config.ADMIN_TOKEN {
        return None;
    }

    Some(AdminUser { authorized: true })
}

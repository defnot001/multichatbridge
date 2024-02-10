use axum::async_trait;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;

#[derive(Debug, Clone)]
pub struct ClientCtx {
    pub client_id: String,
}

impl ClientCtx {
    pub fn new(client_id: String) -> Self {
        Self { client_id }
    }

    pub fn client_id(&self) -> &str {
        &self.client_id
    }
}

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for ClientCtx {
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let client_id = parts
            .headers
            .get("X-Client-ID")
            .and_then(|header| header.to_str().ok())
            .ok_or((StatusCode::UNAUTHORIZED, "Missing X-Client-ID header"))?;

        Ok(Self::new(client_id.to_string()))
    }
}

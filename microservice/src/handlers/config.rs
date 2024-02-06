use axum::{
    extract::Json,
    response::{IntoResponse, Json as JsonResponse},
};
use serde::{Deserialize, Serialize};
use tracing::info;

pub async fn handle_config(body: Json<ConfigBody>) -> impl IntoResponse {
    let body = body.0;
    info!(">>> Received config: {:?}", body);

    JsonResponse(body)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigBody {
    pub subscriptions: Vec<String>,
}

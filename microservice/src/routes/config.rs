use axum::{
    extract::{Json, State},
    response::{IntoResponse, Json as JsonResponse},
};
use serde::{Deserialize, Serialize};

use crate::AppState;

pub async fn handle_config(
    State(app_state): State<AppState>,
    Json(body): Json<ConfigBody>,
) -> impl IntoResponse {
    JsonResponse(body)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigBody {
    pub subscriptions: Vec<String>,
}

use axum::{Json, extract::State, http::StatusCode};
use serde_json::json;

use crate::AppState;

pub async fn health_check(State(state): State<AppState>) -> (StatusCode, Json<serde_json::Value>) {
    let db_status = match sqlx::query("SELECT 1").execute(&state.db).await {
        Ok(_) => "healthy",
        Err(_) => "unhealthy",
    };

    let status_code = if db_status == "healthy" {
        StatusCode::OK
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    };

    (
        status_code,
        Json(json!({
            "status": db_status,
            "message": "Auth server is running"
        })),
    )
}

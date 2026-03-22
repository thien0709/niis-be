use axum::{Json, extract::State, http::StatusCode};

use super::models::{AuthResponse, LoginPayload, RegisterPayload};
use super::service::AuthError;
use crate::state::AppState;

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterPayload>,
) -> Result<Json<AuthResponse>, StatusCode> {
    
    match state.auth_service.register(payload).await {
        Ok(response) => Ok(Json(response)),
        Err(AuthError::UserExists) => Err(StatusCode::BAD_REQUEST),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginPayload>,
) -> Result<Json<AuthResponse>, StatusCode> {
    
    match state.auth_service.login(payload).await {
        Ok(response) => Ok(Json(response)),
        Err(AuthError::InvalidCredentials) => Err(StatusCode::UNAUTHORIZED),
        Err(AuthError::UserBlocked) => Err(StatusCode::FORBIDDEN),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
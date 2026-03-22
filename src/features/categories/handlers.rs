use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use uuid::Uuid;

use super::models::{Category, CreateCategoryPayload, UpdateCategoryPayload};
use super::service::CategoryError; 
use crate::features::auth::Claims;
use crate::state::AppState;

pub async fn create_category(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<CreateCategoryPayload>,
) -> Result<Json<Category>, StatusCode> {
    
    // Đẩy payload sang Service, Handler không cần biết bên trong tạo slug ra sao
    match state.category_service.create_category(&claims, payload).await {
        Ok(category) => Ok(Json(category)),
        Err(CategoryError::Forbidden) => Err(StatusCode::FORBIDDEN),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn get_categories(
    State(state): State<AppState>,
) -> Result<Json<Vec<Category>>, StatusCode> {
    
    match state.category_service.get_categories().await {
        Ok(categories) => Ok(Json(categories)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn update_category(
    State(state): State<AppState>,
    claims: Claims,
    Path(category_id): Path<Uuid>,
    Json(payload): Json<UpdateCategoryPayload>,
) -> Result<Json<Category>, StatusCode> {
    
    match state.category_service.update_category(&claims, category_id, payload).await {
        Ok(category) => Ok(Json(category)),
        Err(CategoryError::Forbidden) => Err(StatusCode::FORBIDDEN),
        Err(CategoryError::NotFound) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn delete_category(
    State(state): State<AppState>,
    claims: Claims,
    Path(category_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    
    match state.category_service.delete_category(&claims, category_id).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(CategoryError::Forbidden) => Err(StatusCode::FORBIDDEN),
        Err(CategoryError::NotFound) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
use axum::{Json, extract::{Path, State}, http::StatusCode};
use uuid::Uuid;

use crate::state::AppState;
use crate::features::auth::Claims;
use super::models::{Post, CreatePostPayload, UpdatePostPayload};

pub async fn create_post(
    State(state): State<AppState>,
    claims: Claims, 
    Json(payload): Json<CreatePostPayload>,
) -> Result<Json<Post>, StatusCode> {
    match state.post_service.create_post(&claims, payload).await {
        Ok(post) => Ok(Json(post)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn get_posts(State(state): State<AppState>) -> Result<Json<Vec<Post>>, StatusCode> {
    match state.post_service.get_all_posts().await {
        Ok(posts) => Ok(Json(posts)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn get_post_by_slug(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<Post>, StatusCode> {
    match state.post_service.get_post_by_slug(&slug).await {
        Ok(post) => Ok(Json(post)),
        Err(e) if e == "Not Found" => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn get_posts_by_category(
    State(state): State<AppState>,
    Path(category_id): Path<Uuid>, 
) -> Result<Json<Vec<Post>>, StatusCode> {
    match state.post_service.get_posts_by_category(category_id).await {
        Ok(posts) => Ok(Json(posts)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn edit_post(
    State(state): State<AppState>,
    claims: Claims,
    Path(post_id): Path<Uuid>, 
    Json(payload): Json<UpdatePostPayload>,
) -> Result<Json<Post>, StatusCode> {
    match state.post_service.edit_post(&claims, post_id, payload).await {
        Ok(post) => Ok(Json(post)),
        Err(e) if e == "Not Found" => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn delete_post(
    State(state): State<AppState>,
    claims: Claims,
    Path(post_id): Path<Uuid>, 
) -> Result<StatusCode, StatusCode> {
    match state.post_service.delete_post(&claims, post_id).await {
        Ok(0) => Err(StatusCode::NOT_FOUND), // Nếu rows_affected = 0
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
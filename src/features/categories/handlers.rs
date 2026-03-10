use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
}; // Thêm Path
use slug::slugify;
use uuid::Uuid;

use super::models::{Category, CreateCategoryPayload, UpdateCategoryPayload};
use crate::features::auth::Claims;
use crate::state::AppState; // // Import từ models

pub async fn create_category(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<CreateCategoryPayload>,
) -> Result<Json<Category>, StatusCode> {
    // 1. KIỂM TRA QUYỀN ADMIN
    if claims.role != "admin" {
        return Err(StatusCode::FORBIDDEN); // Lỗi 403: Không có quyền thực hiện
    }

    // 2. Logic tạo danh mục như cũ
    let slug = slugify(&payload.name);

    let category = sqlx::query_as!(
        Category,
        r#"
        INSERT INTO categories (id, name, slug, description) 
        VALUES (gen_random_uuid(), $1, $2, $3)
        RETURNING id, name, slug, description, created_at
        "#,
        payload.name,
        slug,
        payload.description
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        println!("Lỗi database khi tạo category: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(category))
}

pub async fn get_categories(
    State(state): State<AppState>,
) -> Result<Json<Vec<Category>>, StatusCode> {
    let categories = sqlx::query_as!(
        Category,
        r#"
        SELECT id, name, slug, description, created_at 
        FROM categories
        ORDER BY created_at DESC
        "#
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        println!("Lỗi database khi lấy categories: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(categories))
}
pub async fn update_category(
    State(state): State<AppState>,
    claims: Claims,
    Path(category_id): Path<Uuid>,
    Json(payload): Json<UpdateCategoryPayload>,
) -> Result<Json<Category>, StatusCode> {
    if claims.role != "admin" {
        return Err(StatusCode::FORBIDDEN);
    }

    // Tạo slug mới nếu name được cập nhật
    let new_slug = payload.name.as_ref().map(|name| slugify(name));

    // Thực thi SQL UPDATE và trả về category vừa cập nhật
    let category = sqlx::query_as!(
        Category,
        r#"
        UPDATE categories 
        SET name = COALESCE($1, name), 
            slug = COALESCE($2, slug), 
            description = COALESCE($3, description)
        WHERE id = $4
        RETURNING id, name, slug, description, created_at
        "#,
        payload.name,
        new_slug,
        payload.description,
        category_id,
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        println!("Lỗi database khi cập nhật category: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    match category {
        Some(cat) => Ok(Json(cat)),
        None => Err(StatusCode::NOT_FOUND), // Nếu không tìm thấy category hoặc không phải owner --- IGNORE ---
    }
}

pub async fn delete_category(
    State(state): State<AppState>,
    claims: Claims,
    Path(category_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    
    if claims.role != "admin" {
        return Err(StatusCode::FORBIDDEN);
    }
    // Thực thi SQL DELETE
    let result = sqlx::query!(
        r#"
        DELETE FROM categories 
        WHERE id = $1
        "#,
        category_id,
    )
    .execute(&state.db)
    .await
    .map_err(|e| {
        println!("Lỗi database khi xóa category: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Kiểm tra xem có dòng nào trong database bị xóa thực sự không
    if result.rows_affected() == 0 {
        // Nếu không có dòng nào bị tác động -> Nghĩa là ID sai hoặc người này không có quyền xóa --- IGNORE ---
        return Err(StatusCode::NOT_FOUND);
    }
    Ok(StatusCode::NO_CONTENT)
}

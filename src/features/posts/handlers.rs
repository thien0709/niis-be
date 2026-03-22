use axum::{Json, extract::{Path, State}, http::StatusCode};
use uuid::Uuid;
use slug::slugify;

use crate::state::AppState;
use crate::features::auth::Claims;
use super::models::{Post, CreatePostPayload};


pub async fn create_post(
    State(state): State<AppState>,
    claims: Claims, 
    Json(payload): Json<CreatePostPayload>,
) -> Result<Json<Post>, StatusCode> {
    
    // Tạo ID mới cho bài viết
    let post_id = Uuid::new_v4();

    // Chuyển đổi ID của tác giả (từ token chuỗi sang Uuid)
    let author_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Tự động tạo slug từ tiêu đề
    let slug = slugify(&payload.title);

    let post = sqlx::query_as!(
        Post,
        r#"
        INSERT INTO posts (id, author_id, category_id, title, slug, content_markdown, cover_image_url, published)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id, author_id, category_id, title, slug, content_markdown, cover_image_url, published, created_at, updated_at
        "#,
        post_id,
        author_id,
        payload.category_id, 
        payload.title,
        slug,
        payload.content_markdown,
        payload.cover_image_url,
        false
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        println!("Lỗi database khi tạo post: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(post))
}

pub async fn get_posts(
    State(state): State<AppState>,
) -> Result<Json<Vec<Post>>, StatusCode> {
    
    let posts = sqlx::query_as!(
        Post,
        r#"
        SELECT id, author_id, category_id, title, slug, content_markdown, cover_image_url, published, created_at, updated_at 
        FROM posts
        ORDER BY created_at DESC
        "#
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        println!("Lỗi database khi lấy posts: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(posts))
}

pub async fn get_posts_by_category(
    State(state): State<AppState>,
    Path(category_id): Path<Uuid>, // BẮT LẤY ID TỪ URL BẰNG Path
) -> Result<Json<Vec<Post>>, StatusCode> {
    
    // Dùng SQLx để lọc bài viết bằng mệnh đề WHERE
    let posts = sqlx::query_as!(
        Post,
        r#"
        SELECT id, author_id, category_id, title, slug, content_markdown, cover_image_url, published, created_at, updated_at 
        FROM posts 
        WHERE category_id = $1 
        ORDER BY created_at DESC
        "#,
        category_id
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        println!("Lỗi khi lấy bài viết theo danh mục: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(posts))
}

pub async fn edit_post(
    State(state): State<AppState>,
    claims: Claims,
    Path(post_id): Path<Uuid>, 
    Json(payload): Json<CreatePostPayload>,
) -> Result<Json<Post>, StatusCode> {
    
    // Lấy ID của người đang đăng nhập
    let author_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Tạo slug mới từ title mới
    let new_slug = slugify(&payload.title);

    // Thực thi SQL UPDATE
    let updated_post = sqlx::query_as!(
        Post,
        r#"
        UPDATE posts 
        SET category_id = $1, title = $2, slug = $3, content_markdown = $4, cover_image_url = $5, updated_at = NOW()
        WHERE id = $6 AND author_id = $7
        RETURNING id, author_id, category_id, title, slug, content_markdown, cover_image_url, published, created_at, updated_at
        "#,
        payload.category_id,
        payload.title,
        new_slug,
        payload.content_markdown,
        payload.cover_image_url,
        post_id,    
        author_id   
    )
    .fetch_optional(&state.db) // fetch_optional vì có thể không tìm thấy bài viết
    .await
    .map_err(|e| {
        println!("Lỗi database khi update post: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Kiểm tra kết quả
    match updated_post {
        Some(post) => Ok(Json(post)), 
        None => Err(StatusCode::NOT_FOUND), 
    }
}

pub async fn delete_post(
    State(state): State<AppState>,
    claims: Claims,
    Path(post_id): Path<Uuid>, 
) -> Result<StatusCode, StatusCode> {
    
    // Lấy ID người dùng
    let author_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Thực thi SQL DELETE
    let result = sqlx::query!(
        "DELETE FROM posts WHERE id = $1 AND author_id = $2",
        post_id,
        author_id
    )
    .execute(&state.db)
    .await
    .map_err(|e| {
        println!("Lỗi database khi delete post: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Kiểm tra xem có dòng nào trong database bị xóa thực sự không
    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND); 
    }

    // Trả về mã 204 (No Content) - Dấu hiệu chuẩn của API báo Xóa thành công
    Ok(StatusCode::NO_CONTENT) 
}
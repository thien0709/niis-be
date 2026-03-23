use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
use super::models::{Post, CreatePostPayload, UpdatePostPayload};

#[async_trait]
pub trait PostRepository: Send + Sync {
    async fn create(&self, author_id: Uuid, slug: &str, payload: &CreatePostPayload) -> Result<Post, String>;
    async fn get_all(&self) -> Result<Vec<Post>, String>;
    async fn get_by_slug(&self, slug: &str) -> Result<Option<Post>, String>;
    async fn get_by_category(&self, category_id: Uuid) -> Result<Vec<Post>, String>;
    async fn update(&self, post_id: Uuid, author_id: Uuid, payload: &UpdatePostPayload, new_slug: Option<String>) -> Result<Option<Post>, String>;
    async fn delete(&self, post_id: Uuid, author_id: Uuid) -> Result<u64, String>;
}

pub struct PostgresPostRepository {
    pub pool: PgPool,
}

#[async_trait]
impl PostRepository for PostgresPostRepository {
    async fn create(&self, author_id: Uuid, slug: &str, payload: &CreatePostPayload) -> Result<Post, String> {
        let post_id = Uuid::new_v4();
        sqlx::query_as!(
            Post,
            r#"
            WITH inserted AS (
                INSERT INTO posts (id, author_id, category_id, title, slug, content_markdown, cover_image_url, published)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                RETURNING *
            )
            SELECT i.id, i.author_id, i.category_id, i.title, i.slug, i.content_markdown, i.cover_image_url, i.published, i.created_at, i.updated_at, i.like_count, i.comment_count, i.save_count,
            COALESCE(u.display_name, 'Unknown Author') as "author_name!"
            FROM inserted i LEFT JOIN users u ON i.author_id = u.id
            "#,
            post_id, author_id, payload.category_id, payload.title, slug, payload.content_markdown, payload.cover_image_url, false
        )
        .fetch_one(&self.pool).await.map_err(|e| e.to_string())
    }

    async fn get_all(&self) -> Result<Vec<Post>, String> {
        sqlx::query_as!(
            Post,
            r#"
            SELECT p.id, p.author_id, p.category_id, p.title, p.slug, p.content_markdown, p.cover_image_url, p.published, p.created_at, p.updated_at, p.like_count, p.comment_count, p.save_count,
            COALESCE(u.display_name, 'Unknown Author') as "author_name!"
            FROM posts p LEFT JOIN users u ON p.author_id = u.id ORDER BY p.created_at DESC
            "#
        )
        .fetch_all(&self.pool).await.map_err(|e| e.to_string())
    }

    async fn get_by_slug(&self, slug: &str) -> Result<Option<Post>, String> {
        sqlx::query_as!(
            Post,
            "SELECT p.*, COALESCE(u.display_name, 'Unknown Author') as \"author_name!\" FROM posts p LEFT JOIN users u ON p.author_id = u.id WHERE p.slug = $1",
            slug
        ).fetch_optional(&self.pool).await.map_err(|e| e.to_string())
    }

    async fn get_by_category(&self, category_id: Uuid) -> Result<Vec<Post>, String> {
        sqlx::query_as!(
            Post,
            "SELECT p.*, COALESCE(u.display_name, 'Unknown Author') as \"author_name!\" FROM posts p LEFT JOIN users u ON p.author_id = u.id WHERE p.category_id = $1 ORDER BY p.created_at DESC",
            category_id
        ).fetch_all(&self.pool).await.map_err(|e| e.to_string())
    }

    async fn update(&self, post_id: Uuid, author_id: Uuid, payload: &UpdatePostPayload, new_slug: Option<String>) -> Result<Option<Post>, String> {
        sqlx::query_as!(
            Post,
            r#"
            WITH updated AS (
                UPDATE posts SET category_id = COALESCE($1, category_id), title = COALESCE($2, title), slug = COALESCE($3, slug), content_markdown = COALESCE($4, content_markdown), cover_image_url = COALESCE($5, cover_image_url), published = COALESCE($6, published), updated_at = NOW()
                WHERE id = $7 AND author_id = $8 RETURNING *
            )
            SELECT up.*, COALESCE(u.display_name, 'Unknown Author') as "author_name!" FROM updated up LEFT JOIN users u ON up.author_id = u.id
            "#,
            payload.category_id, payload.title, new_slug, payload.content_markdown, payload.cover_image_url, payload.published, post_id, author_id   
        ).fetch_optional(&self.pool).await.map_err(|e| e.to_string())
    }

    async fn delete(&self, post_id: Uuid, author_id: Uuid) -> Result<u64, String> {
        let result = sqlx::query!("DELETE FROM posts WHERE id = $1 AND author_id = $2", post_id, author_id)
            .execute(&self.pool).await.map_err(|e| e.to_string())?;
        Ok(result.rows_affected())
    }
}
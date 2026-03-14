use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
use super::models::{Category, CreateCategoryPayload, UpdateCategoryPayload};

// 1. ĐỊNH NGHĨA INTERFACE (TRAIT)
#[async_trait]
pub trait CategoryRepository: Send + Sync {
    async fn create(&self, payload: &CreateCategoryPayload, slug: &str) -> Result<Category, String>;
    async fn get_all(&self) -> Result<Vec<Category>, String>;
    async fn update(&self, id: Uuid, payload: &UpdateCategoryPayload, new_slug: Option<&String>) -> Result<Option<Category>, String>;
    async fn delete(&self, id: Uuid) -> Result<u64, String>; // Trả về số dòng bị ảnh hưởng
}

// 2. THỰC THI CHO POSTGRESQL
pub struct PostgresCategoryRepository {
    pub pool: PgPool,
}

#[async_trait]
impl CategoryRepository for PostgresCategoryRepository {
    async fn create(&self, payload: &CreateCategoryPayload, slug: &str) -> Result<Category, String> {
        sqlx::query_as!(
            Category,
            r#"
            INSERT INTO categories (id, name, slug, description) 
            VALUES (gen_random_uuid(), $1, $2, $3)
            RETURNING id, name, slug, description, created_at
            "#,
            payload.name, slug, payload.description
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            println!("Lỗi DB: {}", e);
            "Lỗi Database".to_string()
        })
    }

    async fn get_all(&self) -> Result<Vec<Category>, String> {
        sqlx::query_as!(
            Category,
            "SELECT id, name, slug, description, created_at FROM categories ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|_| "Lỗi Database".to_string())
    }

    async fn update(&self, id: Uuid, payload: &UpdateCategoryPayload, new_slug: Option<&String>) -> Result<Option<Category>, String> {
        sqlx::query_as!(
            Category,
            r#"
            UPDATE categories 
            SET name = COALESCE($1, name), 
                slug = COALESCE($2, slug), 
                description = COALESCE($3, description)
            WHERE id = $4
            RETURNING id, name, slug, description, created_at
            "#,
            payload.name, new_slug, payload.description, id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| "Lỗi Database".to_string())
    }

    async fn delete(&self, id: Uuid) -> Result<u64, String> {
        let result = sqlx::query!("DELETE FROM categories WHERE id = $1", id)
            .execute(&self.pool)
            .await
            .map_err(|_| "Lỗi Database".to_string())?;
            
        Ok(result.rows_affected())
    }
}
use std::sync::Arc;
use uuid::Uuid;
use slug::slugify;
use super::models::{Post, CreatePostPayload, UpdatePostPayload};
use super::repository::PostRepository;
use crate::features::auth::Claims;

#[derive(Clone)]
pub struct PostService {
    repo: Arc<dyn PostRepository>,
}

impl PostService {
    pub fn new(repo: Arc<dyn PostRepository>) -> Self {
        Self { repo }
    }

    pub async fn create_post(&self, claims: &Claims, payload: CreatePostPayload) -> Result<Post, String> {
        let author_id = Uuid::parse_str(&claims.sub).map_err(|_| "Invalid Token".to_string())?;
        let slug = slugify(&payload.title);
        self.repo.create(author_id, &slug, &payload).await
    }

    pub async fn get_all_posts(&self) -> Result<Vec<Post>, String> {
        self.repo.get_all().await
    }

    pub async fn get_post_by_slug(&self, slug: &str) -> Result<Post, String> {
        match self.repo.get_by_slug(slug).await {
            Ok(Some(post)) => Ok(post),
            Ok(None) => Err("Not Found".to_string()),
            Err(e) => Err(e),
        }
    }

    pub async fn get_posts_by_category(&self, category_id: Uuid) -> Result<Vec<Post>, String> {
        self.repo.get_by_category(category_id).await
    }

    pub async fn edit_post(&self, claims: &Claims, post_id: Uuid, payload: UpdatePostPayload) -> Result<Post, String> {
        let author_id = Uuid::parse_str(&claims.sub).map_err(|_| "Invalid Token".to_string())?;
        let new_slug = payload.title.as_ref().map(|t| slugify(t));
        match self.repo.update(post_id, author_id, &payload, new_slug).await {
            Ok(Some(post)) => Ok(post),
            Ok(None) => Err("Not Found".to_string()),
            Err(e) => Err(e),
        }
    }

    pub async fn delete_post(&self, claims: &Claims, post_id: Uuid) -> Result<u64, String> {
        let author_id = Uuid::parse_str(&claims.sub).map_err(|_| "Invalid Token".to_string())?;
        self.repo.delete(post_id, author_id).await
    }
}
use std::sync::Arc;
use uuid::Uuid;
use slug::slugify;
use super::models::{Category, CreateCategoryPayload, UpdateCategoryPayload};
use super::repository::CategoryRepository;
use crate::features::auth::Claims;

#[derive(Clone)]
pub struct CategoryService {
    // Chỉ nắm giữ Interface (Trait)
    repo: Arc<dyn CategoryRepository>,
}

pub enum CategoryError {
    Forbidden,
    NotFound,
    InternalError,
}

impl CategoryService {
    pub fn new(repo: Arc<dyn CategoryRepository>) -> Self {
        Self { repo }
    }

    pub async fn create_category(&self, claims: &Claims, payload: CreateCategoryPayload) -> Result<Category, CategoryError> {
        if claims.role != "admin" { return Err(CategoryError::Forbidden); }
        let slug = slugify(&payload.name);
        
        self.repo.create(&payload, &slug).await.map_err(|_| CategoryError::InternalError)
    }

    pub async fn get_categories(&self) -> Result<Vec<Category>, CategoryError> {
        self.repo.get_all().await.map_err(|_| CategoryError::InternalError)
    }

    pub async fn update_category(&self, claims: &Claims, id: Uuid, payload: UpdateCategoryPayload) -> Result<Category, CategoryError> {
        if claims.role != "admin" { return Err(CategoryError::Forbidden); }
        let new_slug = payload.name.as_ref().map(|n| slugify(n));

        match self.repo.update(id, &payload, new_slug.as_ref()).await {
            Ok(Some(cat)) => Ok(cat),
            Ok(None) => Err(CategoryError::NotFound),
            Err(_) => Err(CategoryError::InternalError),
        }
    }

    pub async fn delete_category(&self, claims: &Claims, id: Uuid) -> Result<(), CategoryError> {
        if claims.role != "admin" { return Err(CategoryError::Forbidden); }

        match self.repo.delete(id).await {
            Ok(0) => Err(CategoryError::NotFound),
            Ok(_) => Ok(()),
            Err(_) => Err(CategoryError::InternalError),
        }
    }
}
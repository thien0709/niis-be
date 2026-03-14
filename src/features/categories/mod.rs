// src/features/categories/mod.rs
pub mod models;
pub mod handlers;
pub mod repository;
pub mod service;

use sqlx::PgPool;
use std::sync::Arc;
use repository::PostgresCategoryRepository;
use service::CategoryService;

pub fn create_category_service(pool: &PgPool) -> CategoryService {
    let repo = PostgresCategoryRepository { pool: pool.clone() };
    let dyn_repo = Arc::new(repo) as Arc<dyn repository::CategoryRepository>;
    CategoryService::new(dyn_repo)
}
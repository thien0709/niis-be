pub mod models;
pub mod handlers;
pub mod repository;
pub mod service;

use sqlx::PgPool;
use std::sync::Arc;
use repository::{CategoryRepository, PostgresCategoryRepository}; // Nhớ import CategoryRepository
use service::CategoryService;

pub fn create_category_service(pool: &PgPool) -> CategoryService {
    let repo = PostgresCategoryRepository { pool: pool.clone() };
    let dyn_repo: Arc<dyn CategoryRepository> = Arc::new(repo);
    
    CategoryService::new(dyn_repo)
}
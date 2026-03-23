pub mod handlers;
pub mod models;
pub mod repository;
pub mod service;

use sqlx::PgPool;
use std::sync::Arc;
use repository::{PostRepository, PostgresPostRepository};
use service::PostService;

pub fn create_post_service(pool: &PgPool) -> PostService {
    let repo = PostgresPostRepository { pool: pool.clone() };
    let dyn_repo: Arc<dyn PostRepository> = Arc::new(repo);
    PostService::new(dyn_repo)
}
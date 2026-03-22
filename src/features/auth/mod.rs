pub mod handlers;
pub mod models;
pub mod jwt;
pub mod guard;
pub mod repository;
pub mod service;

pub use models::Claims; 

use sqlx::PgPool;
use std::sync::Arc;
use repository::{AuthRepository, PostgresAuthRepository};
use service::AuthService;

pub fn create_auth_service(pool: &PgPool) -> AuthService {
    let repo = PostgresAuthRepository { pool: pool.clone() };
    let dyn_repo: Arc<dyn AuthRepository> = Arc::new(repo);
    AuthService::new(dyn_repo)
}
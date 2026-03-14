use sqlx::PgPool;

use crate::features::categories::service::CategoryService;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub category_service: CategoryService,
}
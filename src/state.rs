use crate::features::auth::service::AuthService;
use crate::features::categories::service::CategoryService;

use crate::features::posts::service::PostService;
#[derive(Clone)]
pub struct AppState {
    pub category_service: CategoryService,
    pub post_service: PostService,
    pub auth_service: AuthService,
}

use axum::Router;
use axum::routing::{get, patch, post};
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::env;

mod error;
mod features;
mod state;
mod utils;

use features::auth::handlers::{login, register};
use features::categories::handlers::{
    create_category, delete_category, get_categories, update_category,
};
use features::posts::handlers::{
    create_post, delete_post, edit_post, get_post_by_slug, get_posts, get_posts_by_category,
};
use state::AppState;

use crate::features::categories::create_category_service;
use crate::features::posts::create_post_service;
use crate::features::auth::create_auth_service;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;

    println!("✅ Connected to database!");

    let app_state = AppState {
        category_service: create_category_service(&pool),
        post_service: create_post_service(&pool),
        auth_service: create_auth_service(&pool),
    };

    // Gom tất cả các route hiện tại vào một "nhóm"
    let api_routes = Router::new()
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        .route("/post", post(create_post))
        .route("/post/{id}", patch(edit_post).delete(delete_post)) // Giữ nguyên ngoặc nhọn nhé!
        .route("/posts/{slug}", get(get_post_by_slug))
        .route("/posts", get(get_posts))
        .route("/category", post(create_category))
        .route("/categories", get(get_categories))
        .route(
            "/categories/{id}",
            patch(update_category).delete(delete_category),
        )
        .route("/categories/{id}/posts", get(get_posts_by_category));
    // Bọc nhóm đó vào tiền tố /api/v1 và gắn State
    let app = Router::new()
        .nest("/api/v1", api_routes)
        .with_state(app_state);
    // Chạy server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    println!("🚀 Server running on http://localhost:8080");
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

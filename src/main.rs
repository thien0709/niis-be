use axum::Router;
use axum::routing::{get, post, put};
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
use features::posts::handlers::{create_post, delete_post, edit_post, get_posts, get_posts_by_category};
use state::AppState;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;

    println!("✅ Connected to database!");

    // Tạo AppState chứa pool database
    let app_state = AppState { db: pool };

    // Gom tất cả các route hiện tại vào một "nhóm"
    let api_routes = Router::new()
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        .route("/post", post(create_post))
        .route("/post/{id}", put(edit_post).delete(delete_post)) // Giữ nguyên ngoặc nhọn nhé!
        .route("/posts", get(get_posts))
        .route("/category", post(create_category))
        .route("/categories", get(get_categories))
        .route(
            "/categories/{id}",
            put(update_category).delete(delete_category),
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

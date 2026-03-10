use axum::routing::{get, put, post};
use axum::{Router};
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::env;

mod features;
mod utils;
mod state;
mod error;

use state::AppState;
use features::auth::handlers::{login, register};
use features::posts::handlers::{create_post,edit_post, delete_post, get_posts}; 
use features::categories::handlers::{create_category, get_categories, update_category, delete_category};

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

    // Khởi tạo Router và truyền state vào
    let app = Router::new()
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        .route("/post", post(create_post))
        .route("/post/{id}", put(edit_post).delete(delete_post))
        .route("/posts", get(get_posts))
        .route("/category", post(create_category))
        .route("/categories", get(get_categories))
        .route("/categories/{id}", put(update_category).delete(delete_category))
        .route("/categories/{id}/posts", get(get_posts)) 
        .with_state(app_state);

    // Chạy server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("🚀 Server running on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
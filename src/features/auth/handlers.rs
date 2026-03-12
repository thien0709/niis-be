use axum::{Json, extract::State, http::StatusCode};
use bcrypt::{DEFAULT_COST, hash, verify};

use super::jwt;
// 1. SỬA Ở ĐÂY: Import thêm UserInfo
use super::models::{AuthResponse, LoginPayload, RegisterPayload, UserInfo}; 
use crate::state::AppState;

// =======================
// API ĐĂNG KÝ (/register)
// =======================
pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterPayload>,
) -> Result<Json<AuthResponse>, StatusCode> {
    
    let hashed_password = hash(&payload.password, DEFAULT_COST)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut tx = state.db.begin().await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let user_record = sqlx::query!(
        "INSERT INTO users (username) VALUES ($1) RETURNING id, role, status",
        payload.username
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|_| StatusCode::BAD_REQUEST)?;

    sqlx::query!(
        r#"
        INSERT INTO auth_identities (user_id, provider, provider_user_id, password_hash) 
        VALUES ($1, 'email', $2, $3)
        "#,
        user_record.id,
        payload.username,
        hashed_password
    )
    .execute(&mut *tx)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    tx.commit().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Tạo JWT Token và AuthResponse
    match jwt::create_jwt(&user_record.id.to_string(), &user_record.role) {
        Ok(token) => Ok(Json(AuthResponse { 
            token: token,
            user: UserInfo {
                id: user_record.id.to_string(),
                username: payload.username, 
                role: user_record.role,
                status: user_record.status,
            }
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// =======================
// API ĐĂNG NHẬP (/login)
// =======================
pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginPayload>,
) -> Result<Json<AuthResponse>, StatusCode> {
    
    let record = sqlx::query!(
        r#"
        SELECT u.id, u.role, u.status, ai.password_hash 
        FROM users u 
        JOIN auth_identities ai ON u.id = ai.user_id 
        WHERE ai.provider = 'email' AND ai.provider_user_id = $1
        "#,
        payload.username
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let user = match record {
        Some(u) => u,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    if user.status == "blocked" {
        return Err(StatusCode::FORBIDDEN); 
    }

    let hash_in_db = user.password_hash.unwrap_or_default();
    let is_valid = verify(&payload.password, &hash_in_db).unwrap_or(false);

    if !is_valid {
        return Err(StatusCode::UNAUTHORIZED); 
    }

    // 2. SỬA Ở ĐÂY: Khởi tạo đầy đủ UserInfo cho AuthResponse
    match jwt::create_jwt(&user.id.to_string(), &user.role) {
        Ok(token) => Ok(Json(AuthResponse { 
            token: token,
            user: UserInfo {
                id: user.id.to_string(),
                username: payload.username,
                role: user.role,
                status: user.status,
            }
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
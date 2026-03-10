use axum::{Json, extract::State, http::StatusCode};
use bcrypt::{DEFAULT_COST, hash, verify};

use super::jwt;
use super::models::{AuthResponse, LoginPayload, RegisterPayload};
use crate::state::AppState;

// =======================
// API ĐĂNG KÝ (/register)
// =======================
pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterPayload>,
) -> Result<Json<AuthResponse>, StatusCode> {
    // 1. Băm mật khẩu bằng bcrypt
    let hashed_password =
        hash(&payload.password, DEFAULT_COST).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // 2. Dùng Transaction để đảm bảo nếu lỗi thì rollback cả 2 bảng
    let mut tx = state
        .db
        .begin()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // 3. Insert vào bảng users (Bạn đã viết đúng phần RETURNING id, role ở đây)
    let user_record = sqlx::query!(
        "INSERT INTO users (username) VALUES ($1) RETURNING id, role",
        payload.username
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|_| StatusCode::BAD_REQUEST)?; 

    // 4. Insert vào bảng auth_identities
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

    // 5. Commit transaction
    tx.commit()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // 6. SỬA Ở ĐÂY: Thay chữ "user" cứng bằng user_record.role lấy từ database
    match jwt::create_jwt(&user_record.id.to_string(), &user_record.role) {
        Ok(token) => Ok(Json(AuthResponse { token })),
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
    // 1. SỬA Ở ĐÂY: Lấy thêm u.role và u.status từ database ra
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

    // 2. TÍNH NĂNG MỚI: Kiểm tra xem tài khoản có bị khóa không
    if user.status == "blocked" {
        return Err(StatusCode::FORBIDDEN); // Lỗi 403: Không có quyền truy cập
    }

    // 3. Kiểm tra mật khẩu
    let hash_in_db = user.password_hash.unwrap_or_default();
    let is_valid = verify(&payload.password, &hash_in_db).unwrap_or(false);

    if !is_valid {
        return Err(StatusCode::UNAUTHORIZED); // Mật khẩu sai
    }

    // 4. SỬA Ở ĐÂY: Mật khẩu đúng -> Tạo token với ID và Role lấy từ DB thay vì gắn cứng "user"
    match jwt::create_jwt(&user.id.to_string(), &user.role) {
        Ok(token) => Ok(Json(AuthResponse { token })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts, header::AUTHORIZATION},
};
use super::jwt;    // Gọi file jwt.rs bên cạnh
use super::models::Claims; // Gọi models

impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // 1. Lấy Header
        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .ok_or(StatusCode::UNAUTHORIZED)?;

        // 2. Check "Bearer "
        if !auth_header.starts_with("Bearer ") {
            return Err(StatusCode::UNAUTHORIZED);
        }

        // 3. Cắt token
        let token = &auth_header[7..];

        // 4. Gọi hàm verify từ file jwt.rs
        match jwt::verify_jwt(token) {
            Ok(claims) => Ok(claims),
            Err(_) => Err(StatusCode::UNAUTHORIZED),
        }
    }
}
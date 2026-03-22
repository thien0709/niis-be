use std::sync::Arc;
use bcrypt::{DEFAULT_COST, hash, verify};

use super::repository::AuthRepository;
use super::models::{AuthResponse, LoginPayload, RegisterPayload, UserInfo};
use super::jwt;

#[derive(Clone)]
pub struct AuthService {
    repo: Arc<dyn AuthRepository>,
}

pub enum AuthError {
    InvalidCredentials,
    UserBlocked,
    UserExists,
    InternalError,
}

impl AuthService {
    pub fn new(repo: Arc<dyn AuthRepository>) -> Self {
        Self { repo }
    }

    pub async fn register(&self, payload: RegisterPayload) -> Result<AuthResponse, AuthError> {
        let hashed_password = hash(&payload.password, DEFAULT_COST)
            .map_err(|_| AuthError::InternalError)?;

        let user_record = self.repo.create_user(&payload.username, &hashed_password).await
            .map_err(|_| AuthError::UserExists)?;

        let token = jwt::create_jwt(&user_record.id.to_string(), &user_record.role)
            .map_err(|_| AuthError::InternalError)?;

        Ok(AuthResponse {
            token,
            user: UserInfo {
                id: user_record.id.to_string(),
                username: payload.username,
                role: user_record.role,
                status: user_record.status,
            }
        })
    }

    pub async fn login(&self, payload: LoginPayload) -> Result<AuthResponse, AuthError> {
        let user_opt = self.repo.get_user_by_username(&payload.username).await
            .map_err(|_| AuthError::InternalError)?;

        let user = match user_opt {
            Some(u) => u,
            None => return Err(AuthError::InvalidCredentials),
        };

        if user.status == "blocked" {
            return Err(AuthError::UserBlocked);
        }

        let hash_in_db = user.password_hash.unwrap_or_default();
        let is_valid = verify(&payload.password, &hash_in_db).unwrap_or(false);

        if !is_valid {
            return Err(AuthError::InvalidCredentials);
        }

        let token = jwt::create_jwt(&user.id.to_string(), &user.role)
            .map_err(|_| AuthError::InternalError)?;

        Ok(AuthResponse {
            token,
            user: UserInfo {
                id: user.id.to_string(),
                username: payload.username,
                role: user.role,
                status: user.status,
            }
        })
    }
}
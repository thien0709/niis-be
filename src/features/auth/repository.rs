use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

// Struct phụ để nhận dữ liệu từ DB
pub struct UserLoginRecord {
    pub id: Uuid,
    pub role: String,
    pub status: String,
    pub password_hash: Option<String>,
}

pub struct UserRegisterRecord {
    pub id: Uuid,
    pub role: String,
    pub status: String,
}

#[async_trait]
pub trait AuthRepository: Send + Sync {
    async fn create_user(&self, username: &str, password_hash: &str) -> Result<UserRegisterRecord, String>;
    async fn get_user_by_username(&self, username: &str) -> Result<Option<UserLoginRecord>, String>;
}

pub struct PostgresAuthRepository {
    pub pool: PgPool,
}

#[async_trait]
impl AuthRepository for PostgresAuthRepository {
    async fn create_user(&self, username: &str, password_hash: &str) -> Result<UserRegisterRecord, String> {
        // Mở transaction
        let mut tx = self.pool.begin().await.map_err(|_| "Lỗi bắt đầu transaction".to_string())?;

        let user_record = sqlx::query!(
            "INSERT INTO users (username) VALUES ($1) RETURNING id, role, status",
            username
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|_| "Tài khoản đã tồn tại hoặc lỗi DB".to_string())?;

        sqlx::query!(
            r#"
            INSERT INTO auth_identities (user_id, provider, provider_user_id, password_hash) 
            VALUES ($1, 'email', $2, $3)
            "#,
            user_record.id,
            username,
            password_hash
        )
        .execute(&mut *tx)
        .await
        .map_err(|_| "Lỗi lưu thông tin mật khẩu".to_string())?;

        // Commit transaction
        tx.commit().await.map_err(|_| "Lỗi commit transaction".to_string())?;

        Ok(UserRegisterRecord {
            id: user_record.id,
            role: user_record.role,
            status: user_record.status,
        })
    }

    async fn get_user_by_username(&self, username: &str) -> Result<Option<UserLoginRecord>, String> {
        let record = sqlx::query_as!(
            UserLoginRecord,
            r#"
            SELECT u.id, u.role, u.status, ai.password_hash 
            FROM users u 
            JOIN auth_identities ai ON u.id = ai.user_id 
            WHERE ai.provider = 'email' AND ai.provider_user_id = $1
            "#,
            username
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| "Lỗi database".to_string())?;

        Ok(record)
    }
}
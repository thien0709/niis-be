use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey, errors::Error};
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};
use super::models::Claims; // Import Claims từ file bên cạnh

// Hàm tạo Token
pub fn create_jwt(user_id: &str, role: &str) -> Result<String, Error> {
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    
    let expiration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize + (24 * 3600); // 1 ngày

    let claims = Claims {
        sub: user_id.to_string(),
        role: role.to_string(),
        exp: expiration,
        iat: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

// Hàm giải mã Token (Dùng cho Guard)
pub fn verify_jwt(token: &str) -> Result<Claims, Error> {
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;
    
    Ok(token_data.claims)
}



pub mod admin;
pub mod user;

use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey, errors::Error as JwtError};
use serde::{Deserialize, Serialize};

/// Chave usada para assinar e verificar tokens JWT.
/// Em produção, deveria vir de uma variável de ambiente.
const JWT_SECRET_ENV: &str = "JWT_SECRET";

/// Claims customizados armazenados dentro do JWT.
/// Contém o ID e username do usuário autenticado.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UserClaims {
    pub user_id: i64,
    pub username: String,
    pub exp: usize, // Necessário para expiração
}

/// Gera um token JWT válido por 24 horas para o usuário.
pub fn generate_token(user_id: i64, username: &str) -> Result<String, JwtError> {
    let secret = std::env::var(JWT_SECRET_ENV).unwrap_or_else(|_| "default-secret".to_string());
    
    // Expira em 24h
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::TimeDelta::try_hours(24).unwrap())
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = UserClaims {
        user_id,
        username: username.to_string(),
        exp: expiration,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes())
    )
}

/// Verifica um token JWT e retorna os claims do usuário.
pub fn verify_token(token: &str) -> Result<UserClaims, JwtError> {
    let secret = std::env::var(JWT_SECRET_ENV).unwrap_or_else(|_| "default-secret".to_string());
    
    let token_data = decode::<UserClaims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default() // Valida expiração por padrão
    )?;

    Ok(token_data.claims)
}

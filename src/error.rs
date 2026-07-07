use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Serialize;
use thiserror::Error;

/// Enum centralizado de erros da aplicação.
/// Usa `thiserror` para derivar Display e Error automaticamente.
/// Cada variante mapeia para um StatusCode HTTP específico.
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Cabeçalho de autorização ausente")]
    MissingAuthorization,

    #[error("Credenciais inválidas")]
    InvalidCredentials,

    #[error("Ativo não encontrado")]
    AssetDoesNotExist,

    #[error("Usuário não encontrado")]
    UserDoesNotExist,

    #[error("Este nome de usuário já está em uso")]
    UsernameTaken,

    // ── Melhorias: Validações customizadas ──
    #[error("Nome do ativo não pode ser vazio")]
    InvalidAssetName,

    #[error("Valor unitário deve ser maior que zero")]
    InvalidValue,

    #[error("Quantidade deve ser maior que zero")]
    InvalidQuantity,

    #[error("As senhas não coincidem")]
    PasswordMismatch,

    #[error("A senha deve ter pelo menos 6 caracteres")]
    PasswordTooShort,

    #[error("O nome de usuário deve ter pelo menos 3 caracteres")]
    UsernameTooShort,

    // ── Erros de infraestrutura (transparentes) ──
    #[error(transparent)]
    Database(#[from] sqlx::Error),

    #[error(transparent)]
    Template(#[from] askama::Error),
}

/// Resposta de erro serializada como JSON.
#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

/// Implementação de IntoResponse para converter AppError em resposta HTTP.
/// Isso permite usar AppError como tipo de retorno em handlers do Axum.
impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let status = match &self {
            AppError::MissingAuthorization => StatusCode::UNAUTHORIZED,
            AppError::InvalidCredentials => StatusCode::UNAUTHORIZED,
            AppError::AssetDoesNotExist => StatusCode::NOT_FOUND,
            AppError::UserDoesNotExist => StatusCode::NOT_FOUND,
            AppError::UsernameTaken => StatusCode::CONFLICT,
            AppError::InvalidAssetName
            | AppError::InvalidValue
            | AppError::InvalidQuantity
            | AppError::PasswordMismatch
            | AppError::PasswordTooShort
            | AppError::UsernameTooShort => StatusCode::BAD_REQUEST,
            AppError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Template(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let body = Json(ErrorResponse {
            error: self.to_string(),
        });

        (status, body).into_response()
    }
}

use axum::extract::FromRequestParts;

use crate::{app::AppState, auth::UserClaims, error::AppError};

/// Extrator de autenticação para usuários comuns.
/// Verifica se o cookie contém um JWT válido.
/// Uso: `async fn handler(user: AuthenticatedUser) -> ... { ... }`
pub struct AuthenticatedUser {
    pub claims: UserClaims,
}

impl FromRequestParts<AppState> for AuthenticatedUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // Extrai o jar de cookies assinados
        let jar = axum_extra::extract::SignedCookieJar::<axum_extra::extract::cookie::Key>::from_request_parts(parts, state)
            .await
            .map_err(|_| AppError::MissingAuthorization)?;

        // Busca o cookie "token"
        let token_cookie = jar
            .get("token")
            .ok_or(AppError::MissingAuthorization)?;

        // Verifica o JWT
        let claims = crate::auth::verify_token(token_cookie.value())
            .map_err(|_| AppError::InvalidCredentials)?;

        Ok(AuthenticatedUser { claims })
    }
}

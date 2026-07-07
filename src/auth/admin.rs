use axum::extract::FromRequestParts;

use crate::{app::AppState, auth::UserClaims, error::AppError};

/// Extrator de autenticação para administradores.
/// Nesta versão, funciona igual ao AuthenticatedUser.
/// Pode ser expandido com verificação de papel/role futuramente.
pub struct AdminUser {
    pub claims: UserClaims,
}

impl FromRequestParts<AppState> for AdminUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let jar = axum_extra::extract::SignedCookieJar::<axum_extra::extract::cookie::Key>::from_request_parts(parts, state)
            .await
            .map_err(|_| AppError::MissingAuthorization)?;

        let token_cookie = jar
            .get("token")
            .ok_or(AppError::MissingAuthorization)?;

        let claims = crate::auth::verify_token(token_cookie.value())
            .map_err(|_| AppError::InvalidCredentials)?;

        Ok(AdminUser { claims })
    }
}

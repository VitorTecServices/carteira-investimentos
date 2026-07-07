use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
    routing::{delete, get, post, put},
    Router,
};
use axum_extra::extract::{cookie::Cookie, SignedCookieJar};
use password_auth::{generate_hash, verify_password};

use crate::{
    app::AppState,
    auth::{generate_token, user::AuthenticatedUser},
    error::AppError,
    models::{Asset, AssetInput, LoginInput, PortfolioInput, RegisterInput},
    repository::Repository,
};

/// Registra um novo usuário no sistema.
async fn register(
    State(_state): State<AppState>,
    repo: Repository,
    Json(input): Json<RegisterInput>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    // Validações
    if input.username.len() < 3 {
        return Err(AppError::UsernameTooShort);
    }
    if input.password.len() < 6 {
        return Err(AppError::PasswordTooShort);
    }
    if input.password != input.password_confirm {
        return Err(AppError::PasswordMismatch);
    }
    if repo.find_user_by_username(&input.username).await?.is_some() {
        return Err(AppError::UsernameTaken);
    }

    // Gera o hash da senha
    let password_hash = generate_hash(&input.password);

    // Salva no banco de dados
    repo.create_user(input.username, password_hash).await?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({ "message": "Usuário criado com sucesso" })),
    ))
}

/// Realiza login e retorna um cookie assinado com o JWT.
async fn login(
    State(_state): State<AppState>,
    repo: Repository,
    jar: SignedCookieJar,
    Json(input): Json<LoginInput>,
) -> Result<(SignedCookieJar, Json<serde_json::Value>), AppError> {
    // Busca usuário pelo username
    let user = repo
        .find_user_by_username(&input.username)
        .await?
        .ok_or(AppError::InvalidCredentials)?;

    // Verifica a senha
    verify_password(&input.password, &user.password_hash)
        .map_err(|_| AppError::InvalidCredentials)?;

    // Gera o token JWT
    let token = generate_token(user.id, &user.username).map_err(|_| AppError::InvalidCredentials)?;

    // Configura o cookie assinado (HttpOnly é recomendado em prod)
    let cookie = Cookie::build(("token", token))
        .path("/")
        .http_only(true)
        .build();

    let jar = jar.add(cookie);

    Ok((
        jar,
        Json(serde_json::json!({ "message": "Login efetuado com sucesso" })),
    ))
}

/// Realiza logout removendo o cookie.
async fn logout(jar: SignedCookieJar) -> (SignedCookieJar, Json<serde_json::Value>) {
    let jar = jar.remove(Cookie::from("token"));
    (
        jar,
        Json(serde_json::json!({ "message": "Logout efetuado" })),
    )
}

// ── Rotas de Assets (Ativos) ──

async fn list_assets(repo: Repository) -> Result<Json<Vec<Asset>>, AppError> {
    let assets = repo.list_assets().await?;
    Ok(Json(assets))
}

async fn create_asset(
    _user: AuthenticatedUser, // Requer autenticação
    repo: Repository,
    Json(input): Json<AssetInput>,
) -> Result<(StatusCode, Json<Asset>), AppError> {
    if input.name.trim().is_empty() {
        return Err(AppError::InvalidAssetName);
    }
    if input.unit_value <= 0.0 {
        return Err(AppError::InvalidValue);
    }

    let asset = repo.create_asset(input.name, input.unit_value).await?;
    Ok((StatusCode::CREATED, Json(asset)))
}

async fn update_asset(
    _user: AuthenticatedUser,
    repo: Repository,
    Path(id): Path<i64>,
    Json(input): Json<AssetInput>,
) -> Result<Json<Asset>, AppError> {
    if input.name.trim().is_empty() {
        return Err(AppError::InvalidAssetName);
    }
    if input.unit_value <= 0.0 {
        return Err(AppError::InvalidValue);
    }

    // Verifica se existe
    let _ = repo.get_asset(id).await?.ok_or(AppError::AssetDoesNotExist)?;

    let asset = repo.update_asset(id, input.name, input.unit_value).await?;
    Ok(Json(asset))
}

async fn delete_asset(
    _user: AuthenticatedUser,
    repo: Repository,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    // Verifica se existe
    let _ = repo.get_asset(id).await?.ok_or(AppError::AssetDoesNotExist)?;
    
    repo.delete_asset(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ── Rotas de Portfólio (Usuário Logado) ──

async fn list_portfolio(
    user: AuthenticatedUser,
    repo: Repository,
) -> Result<Json<serde_json::Value>, AppError> {
    let items = repo.list_portfolio_items(user.claims.user_id).await?;
    let total = repo.get_portfolio_total(user.claims.user_id).await?;
    
    Ok(Json(serde_json::json!({
        "items": items,
        "total_value": total
    })))
}

async fn add_to_portfolio(
    user: AuthenticatedUser,
    repo: Repository,
    Json(input): Json<PortfolioInput>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    if input.quantity <= 0.0 {
        return Err(AppError::InvalidQuantity);
    }
    
    // Verifica se o ativo existe
    let _ = repo.get_asset(input.asset_id).await?.ok_or(AppError::AssetDoesNotExist)?;

    let item = repo.add_portfolio_item(user.claims.user_id, input.asset_id, input.quantity).await?;
    Ok((StatusCode::CREATED, Json(serde_json::json!(item))))
}

async fn remove_from_portfolio(
    user: AuthenticatedUser,
    repo: Repository,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    repo.remove_portfolio_item(id, user.claims.user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Cria o roteador da API.
pub fn router() -> Router<AppState> {
    Router::new()
        // Rotas públicas de autenticação
        .route("/api/register", post(register))
        .route("/api/login", post(login))
        .route("/api/logout", post(logout))
        
        // Rotas de listagem pública (opcional, pode ser protegida)
        .route("/api/assets", get(list_assets))
        
        // Rotas que exigem autenticação para modificar dados
        .route("/api/assets", post(create_asset))
        .route("/api/assets/{id}", put(update_asset).delete(delete_asset))
        
        // Rotas específicas do usuário logado
        .route("/api/portfolio", get(list_portfolio).post(add_to_portfolio))
        .route("/api/portfolio/{id}", delete(remove_from_portfolio))
}

use askama::Template;
use axum::{
    extract::Path,
    response::{Html, IntoResponse, Redirect},
    routing::get,
    Router,
};
use axum_extra::extract::SignedCookieJar;

use crate::{
    app::AppState,
    auth::{user::AuthenticatedUser, verify_token},
    error::AppError,
    models::{Asset, PortfolioItemView},
    repository::Repository,
};

// ── Templates ──

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate;

#[derive(Template)]
#[template(path = "register.html")]
struct RegisterTemplate;

#[derive(Template)]
#[template(path = "dashboard.html")]
struct DashboardTemplate {
    username: String,
    portfolio_items: Vec<PortfolioItemView>,
    total_value: f64,
}

#[derive(Template)]
#[template(path = "asset_form.html")]
#[allow(dead_code)]
struct AssetFormTemplate {
    username: String,
    asset: Option<Asset>, // Se None, é criação. Se Some, é edição.
    assets_list: Vec<Asset>, // Usado para adicionar no portfólio
}

// ── Handlers ──

/// Verifica se o usuário já tem token válido. Se sim, redireciona pro dashboard.
async fn check_redirect_to_dashboard(jar: &SignedCookieJar) -> Option<Redirect> {
    if let Some(cookie) = jar.get("token") {
        if verify_token(cookie.value()).is_ok() {
            return Some(Redirect::to("/dashboard"));
        }
    }
    None
}

async fn login_page(jar: SignedCookieJar) -> Result<impl IntoResponse, AppError> {
    if let Some(redirect) = check_redirect_to_dashboard(&jar).await {
        return Ok(redirect.into_response());
    }
    let template = LoginTemplate;
    Ok(Html(template.render()?).into_response())
}

async fn register_page(jar: SignedCookieJar) -> Result<impl IntoResponse, AppError> {
    if let Some(redirect) = check_redirect_to_dashboard(&jar).await {
        return Ok(redirect.into_response());
    }
    let template = RegisterTemplate;
    Ok(Html(template.render()?).into_response())
}

async fn dashboard_page(
    user: AuthenticatedUser,
    repo: Repository,
) -> Result<Html<String>, AppError> {
    // Busca itens do portfólio do usuário
    let items = repo.list_portfolio_items(user.claims.user_id).await?;
    let total = repo.get_portfolio_total(user.claims.user_id).await?;

    let template = DashboardTemplate {
        username: user.claims.username,
        portfolio_items: items,
        total_value: total,
    };
    
    Ok(Html(template.render()?))
}

async fn new_asset_page(
    user: AuthenticatedUser,
    repo: Repository,
) -> Result<Html<String>, AppError> {
    let assets = repo.list_assets().await?;
    
    let template = AssetFormTemplate {
        username: user.claims.username,
        asset: None,
        assets_list: assets,
    };
    Ok(Html(template.render()?))
}

async fn edit_asset_page(
    user: AuthenticatedUser,
    repo: Repository,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let asset = repo.get_asset(id).await?;
    let assets = repo.list_assets().await?;
    
    if asset.is_none() {
        return Ok(Redirect::to("/dashboard").into_response());
    }

    let template = AssetFormTemplate {
        username: user.claims.username,
        asset,
        assets_list: assets,
    };
    Ok(Html(template.render()?).into_response())
}

/// Cria o roteador do frontend.
pub fn router() -> Router<AppState> {
    Router::new()
        // Rotas públicas ou que fazem redirecionamento
        .route("/", get(|| async { Redirect::to("/login") }))
        .route("/login", get(login_page))
        .route("/register", get(register_page))
        
        // Rotas protegidas pelas views (requerem AuthenticatedUser)
        .route("/dashboard", get(dashboard_page))
        .route("/assets/new", get(new_asset_page))
        .route("/assets/{id}/edit", get(edit_asset_page))
}

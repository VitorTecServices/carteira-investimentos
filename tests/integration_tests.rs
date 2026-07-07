/// Testes de integração da aplicação Carteira de Investimentos.
/// Requer o banco PostgreSQL rodando (docker compose up -d).
///
/// Executar: cargo test -- --test-threads=1
use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use serde_json::json;
use sqlx::PgPool;
use tower::ServiceExt;

use wallet_live::{auth, routes};

/// Helper para construir o app Router com estado real do banco.
async fn setup_app() -> (axum::Router, PgPool) {
    let _ = dotenvy::dotenv();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://carteira_admin:carteira_pass_2026@localhost:5433/carteira_investimentos".to_string());

    let db = PgPool::connect(&database_url)
        .await
        .expect("Falha ao conectar ao banco. Certifique-se de que 'docker compose up -d' foi executado.");

    // Executa migrations
    sqlx::migrate!("./migrations")
        .run(&db)
        .await
        .expect("Falha ao rodar migrations");

    let cookie_secret = std::env::var("COOKIE_SECRET")
        .unwrap_or_else(|_| "012345678901234567890123456789012345678901234567890123456789".to_string());
    
    let mut secret = cookie_secret.into_bytes();
    while secret.len() < 64 {
        secret.push(0);
    }
    let key = axum_extra::extract::cookie::Key::from(&secret[0..64]);

    let state = wallet_live::app::AppState { db: db.clone(), key };
    let app = routes::router().with_state(state);

    (app, db)
}

/// Limpa dados de teste para garantir isolamento entre testes.
async fn cleanup(db: &PgPool) {
    sqlx::query("DELETE FROM portfolio_items").execute(db).await.ok();
    sqlx::query("DELETE FROM assets").execute(db).await.ok();
    sqlx::query("DELETE FROM users WHERE username LIKE 'test_%'").execute(db).await.ok();
}

// ══════════════════════════════════════════════════════════════════════
// Testes Unitários — Módulo auth (JWT)
// ══════════════════════════════════════════════════════════════════════

#[test]
fn test_generate_and_verify_token() {
    // SAFETY: teste roda single-threaded (--test-threads=1)
    unsafe { std::env::set_var("JWT_SECRET", "test-secret-key-123") };

    let token = auth::generate_token(42, "testuser")
        .expect("Token deveria ser gerado com sucesso");

    let claims = auth::verify_token(&token)
        .expect("Token deveria ser verificado com sucesso");

    assert_eq!(claims.user_id, 42);
    assert_eq!(claims.username, "testuser");
}

#[test]
fn test_verify_invalid_token() {
    // SAFETY: teste roda single-threaded (--test-threads=1)
    unsafe { std::env::set_var("JWT_SECRET", "test-secret-key-123") };

    let result = auth::verify_token("token.invalido.aqui");
    assert!(result.is_err(), "Token inválido deveria retornar erro");
}

#[test]
fn test_token_contains_expiration() {
    // SAFETY: teste roda single-threaded (--test-threads=1)
    unsafe { std::env::set_var("JWT_SECRET", "test-secret-key-123") };

    let token = auth::generate_token(1, "user1").unwrap();
    let claims = auth::verify_token(&token).unwrap();

    // Expiration deve ser no futuro (24h a partir de agora)
    let now = chrono::Utc::now().timestamp() as usize;
    assert!(claims.exp > now, "Token deveria expirar no futuro");
    
    // Deve expirar entre 23h e 25h a partir de agora (margem)
    let twenty_three_hours = now + (23 * 3600);
    let twenty_five_hours = now + (25 * 3600);
    assert!(claims.exp > twenty_three_hours && claims.exp < twenty_five_hours);
}

// ══════════════════════════════════════════════════════════════════════
// Testes de Integração — API
// ══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_register_user_success() {
    let (app, db) = setup_app().await;
    cleanup(&db).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/register")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "username": "test_user_register",
                        "password": "senha123",
                        "password_confirm": "senha123"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    // Limpeza
    sqlx::query("DELETE FROM users WHERE username = 'test_user_register'")
        .execute(&db)
        .await
        .ok();
}

#[tokio::test]
async fn test_register_username_too_short() {
    let (app, db) = setup_app().await;
    cleanup(&db).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/register")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "username": "ab",
                        "password": "senha123",
                        "password_confirm": "senha123"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_register_password_mismatch() {
    let (app, db) = setup_app().await;
    cleanup(&db).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/register")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "username": "test_mismatch",
                        "password": "senha123",
                        "password_confirm": "outra456"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_login_invalid_credentials() {
    let (app, db) = setup_app().await;
    cleanup(&db).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/login")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "username": "nao_existe",
                        "password": "qualquer"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_list_assets_public() {
    let (app, _db) = setup_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/assets")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_create_asset_without_auth() {
    let (app, _db) = setup_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/assets")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "name": "BTC",
                        "unit_value": 350000.0
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // Sem autenticação, deve retornar 401
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_dashboard_without_auth() {
    let (app, _db) = setup_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/dashboard")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Sem cookie, deve retornar 401
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_login_page_returns_html() {
    let (app, _db) = setup_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/login")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_register_page_returns_html() {
    let (app, _db) = setup_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/register")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_root_redirects_to_login() {
    let (app, _db) = setup_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Redirect 303 ou 307
    assert!(
        response.status().is_redirection(),
        "Root deveria redirecionar, got: {}",
        response.status()
    );
}

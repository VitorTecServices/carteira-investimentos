use axum::extract::FromRef;
use axum_extra::extract::cookie::Key;
use sqlx::PgPool;
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::{
    fmt::format::FmtSpan, layer::SubscriberExt, util::SubscriberInitExt,
};

use crate::routes;

/// Estado compartilhado da aplicação, passado para os handlers.
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub key: Key, // Chave para assinar cookies
}

impl FromRef<AppState> for Key {
    fn from_ref(state: &AppState) -> Self {
        state.key.clone()
    }
}


impl AppState {
    /// Inicializa o estado lendo as variáveis de ambiente e conectando ao banco.
    async fn new() -> color_eyre::Result<Self> {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://carteira_admin:carteira_pass_2026@localhost:5433/carteira_investimentos".to_string());
        
        info!("Conectando ao banco de dados: {}", database_url);
        let db = PgPool::connect(&database_url).await?;

        // Lendo chave do cookie
        let cookie_secret = std::env::var("COOKIE_SECRET")
            .unwrap_or_else(|_| "012345678901234567890123456789012345678901234567890123456789".to_string());
        
        let mut secret = cookie_secret.into_bytes();
        while secret.len() < 64 {
            secret.push(0);
        }
        let key = Key::from(&secret[0..64]);

        Ok(Self { db, key })
    }
}

pub struct App;

impl App {
    /// Inicia o servidor Axum.
    pub async fn start() -> color_eyre::Result<()> {
        // Inicializa o sistema de logs (tracing)
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "wallet_live=debug,info".into()),
            )
            .with(
                tracing_subscriber::fmt::layer()
                    .with_span_events(FmtSpan::CLOSE)
                    .compact(),
            )
            .init();

        info!("Inicializando AppState e Banco de Dados...");
        let state = AppState::new().await?;

        info!("Rodando migrations...");
        sqlx::migrate!("./migrations")
            .run(&state.db)
            .await?;

        info!("Construindo rotas...");
        let app = routes::router().with_state(state);

        let addr = "0.0.0.0:3030";
        info!("Servidor escutando em http://{}", addr);
        let listener = TcpListener::bind(addr).await?;
        
        axum::serve(listener, app).await?;

        Ok(())
    }
}

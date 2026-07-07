pub mod api;
pub mod frontend;

use axum::Router;
use crate::app::AppState;

/// Agrupa todas as rotas (API + Frontend)
pub fn router() -> Router<AppState> {
    Router::new()
        .merge(frontend::router())
        .merge(api::router())
}

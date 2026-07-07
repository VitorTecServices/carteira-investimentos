use std::convert::Infallible;

use axum::extract::FromRequestParts;
use sqlx::PgPool;

use crate::{
    app::AppState,
    models::{Asset, PortfolioItem, PortfolioItemView, UserRecord},
};

/// Repository encapsula o acesso ao banco de dados.
/// É extraído automaticamente das requisições pelo Axum.
pub struct Repository {
    db: PgPool,
}

impl Repository {
    // ── Assets (Ativos) ──

    /// Lista todos os ativos cadastrados no sistema.
    pub async fn list_assets(&self) -> sqlx::Result<Vec<Asset>> {
        sqlx::query_as!(
            Asset,
            "SELECT id, name, unit_value
             FROM assets
             ORDER BY name;"
        )
        .fetch_all(&self.db)
        .await
    }

    /// Busca um ativo pelo ID.
    pub async fn get_asset(&self, id: i64) -> sqlx::Result<Option<Asset>> {
        sqlx::query_as!(
            Asset,
            "SELECT id, name, unit_value
             FROM assets
             WHERE id = $1;",
            id
        )
        .fetch_optional(&self.db)
        .await
    }

    /// Cria um novo ativo com nome e valor unitário.
    pub async fn create_asset(&self, name: String, unit_value: f64) -> sqlx::Result<Asset> {
        sqlx::query_as!(
            Asset,
            "INSERT INTO assets (name, unit_value)
             VALUES ($1, $2)
             RETURNING id, name, unit_value;",
            name,
            unit_value
        )
        .fetch_one(&self.db)
        .await
    }

    /// Atualiza o nome e valor unitário de um ativo existente.
    pub async fn update_asset(
        &self,
        id: i64,
        name: String,
        unit_value: f64,
    ) -> sqlx::Result<Asset> {
        sqlx::query_as!(
            Asset,
            "UPDATE assets
             SET name = $1, unit_value = $2
             WHERE id = $3
             RETURNING id, name, unit_value;",
            name,
            unit_value,
            id
        )
        .fetch_one(&self.db)
        .await
    }

    /// Remove um ativo pelo ID. Melhoria: permite exclusão de ativos.
    pub async fn delete_asset(&self, id: i64) -> sqlx::Result<()> {
        sqlx::query!("DELETE FROM assets WHERE id = $1;", id)
            .execute(&self.db)
            .await?;
        Ok(())
    }

    // ── Users (Usuários) ──

    /// Busca um usuário pelo username.
    pub async fn find_user_by_username(
        &self,
        username: &str,
    ) -> sqlx::Result<Option<UserRecord>> {
        sqlx::query_as!(
            UserRecord,
            "SELECT id, username, password_hash
             FROM users
             WHERE username = $1;",
            username
        )
        .fetch_optional(&self.db)
        .await
    }

    /// Cria um novo usuário com username e hash da senha.
    pub async fn create_user(
        &self,
        username: String,
        password_hash: String,
    ) -> sqlx::Result<UserRecord> {
        sqlx::query_as!(
            UserRecord,
            "INSERT INTO users (username, password_hash)
             VALUES ($1, $2)
             RETURNING id, username, password_hash;",
            username,
            password_hash
        )
        .fetch_one(&self.db)
        .await
    }

    // ── Portfolio (Portfólio) — Melhoria ──

    /// Adiciona um ativo ao portfólio do usuário.
    /// Se o ativo já existe no portfólio, atualiza a quantidade.
    pub async fn add_portfolio_item(
        &self,
        user_id: i64,
        asset_id: i64,
        quantity: f64,
    ) -> sqlx::Result<PortfolioItem> {
        sqlx::query_as!(
            PortfolioItem,
            "INSERT INTO portfolio_items (user_id, asset_id, quantity)
             VALUES ($1, $2, $3)
             ON CONFLICT (user_id, asset_id)
             DO UPDATE SET quantity = portfolio_items.quantity + $3
             RETURNING id, user_id, asset_id, quantity;",
            user_id,
            asset_id,
            quantity
        )
        .fetch_one(&self.db)
        .await
    }

    /// Lista todos os itens do portfólio de um usuário com dados do ativo.
    pub async fn list_portfolio_items(
        &self,
        user_id: i64,
    ) -> sqlx::Result<Vec<PortfolioItemView>> {
        sqlx::query_as!(
            PortfolioItemView,
            "SELECT
                p.id as portfolio_id,
                a.name as asset_name,
                a.unit_value,
                p.quantity,
                (a.unit_value * p.quantity) as \"total_value!\"
             FROM portfolio_items p
             JOIN assets a ON a.id = p.asset_id
             WHERE p.user_id = $1
             ORDER BY a.name;",
            user_id
        )
        .fetch_all(&self.db)
        .await
    }

    /// Remove um item do portfólio do usuário.
    pub async fn remove_portfolio_item(
        &self,
        portfolio_id: i64,
        user_id: i64,
    ) -> sqlx::Result<()> {
        sqlx::query!(
            "DELETE FROM portfolio_items
             WHERE id = $1 AND user_id = $2;",
            portfolio_id,
            user_id
        )
        .execute(&self.db)
        .await?;
        Ok(())
    }

    /// Calcula o valor total do portfólio de um usuário.
    pub async fn get_portfolio_total(&self, user_id: i64) -> sqlx::Result<f64> {
        let result: Option<f64> = sqlx::query_scalar!(
            "SELECT COALESCE(SUM(a.unit_value * p.quantity), 0.0)
             FROM portfolio_items p
             JOIN assets a ON a.id = p.asset_id
             WHERE p.user_id = $1;",
            user_id
        )
        .fetch_one(&self.db)
        .await?;
        Ok(result.unwrap_or(0.0))
    }
}

/// Extrator do Axum: permite injetar Repository diretamente nos handlers.
/// Exemplo: `async fn handler(repo: Repository) -> ... { ... }`
impl FromRequestParts<AppState> for Repository {
    type Rejection = Infallible;

    async fn from_request_parts(
        _parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Ok(Self {
            db: state.db.clone(),
        })
    }
}

use serde::{Deserialize, Serialize};

/// Representa um ativo de investimento cadastrado no sistema.
/// Cada ativo tem um nome único e um valor unitário.
#[derive(Serialize, Clone, Debug)]
pub struct Asset {
    pub id: i64,
    pub name: String,
    pub unit_value: f64,
}

/// Registro de usuário armazenado no banco de dados.
/// A senha é armazenada como hash (nunca em texto puro).
#[derive(Clone, Debug)]
pub struct UserRecord {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
}

/// Item do portfólio — vincula um ativo a um usuário com uma quantidade.
/// Melhoria: permite que cada usuário tenha sua própria carteira.
#[derive(Serialize, Clone, Debug)]
pub struct PortfolioItem {
    pub id: i64,
    pub user_id: i64,
    pub asset_id: i64,
    pub quantity: f64,
}

/// View do portfólio para exibição no dashboard.
/// Combina dados do ativo com a quantidade do usuário e calcula o valor total.
#[derive(Serialize, Clone, Debug)]
pub struct PortfolioItemView {
    pub portfolio_id: i64,
    pub asset_name: String,
    pub unit_value: f64,
    pub quantity: f64,
    pub total_value: f64,
}

// ── Structs de entrada (deserialize) para receber dados das requisições ──

/// Dados para criar ou atualizar um ativo.
#[derive(Deserialize, Debug)]
pub struct AssetInput {
    pub name: String,
    pub unit_value: f64,
}

/// Dados de login do usuário.
#[derive(Deserialize, Debug)]
pub struct LoginInput {
    pub username: String,
    pub password: String,
}

/// Dados de registro de novo usuário.
#[derive(Deserialize, Debug)]
pub struct RegisterInput {
    pub username: String,
    pub password: String,
    pub password_confirm: String,
}

/// Dados para adicionar um ativo ao portfólio.
#[derive(Deserialize, Debug)]
pub struct PortfolioInput {
    pub asset_id: i64,
    pub quantity: f64,
}

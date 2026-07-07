use wallet_live::app::App;

/// Carteira de Investimentos Fullstack
/// Desenvolvido por VitorTechServices
///
/// Ponto de entrada do executável.
/// Inicializa o color_eyre para melhores mensagens de erro e chama a App.
#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    // Tenta carregar variáveis de ambiente do .env se existir
    let _ = dotenvy::dotenv();

    // Inicializa color_eyre para formatação bonita de pânicos/erros
    color_eyre::install()?;

    App::start().await?;

    Ok(())
}

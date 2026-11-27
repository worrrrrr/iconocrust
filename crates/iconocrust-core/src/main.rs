use std::net::SocketAddr;

use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    let address = SocketAddr::from(([127, 0, 0, 1], 7432));
    let listener = TcpListener::bind(address).await?;
    tracing::info!(%address, "Iconocrust Core is listening");

    axum::serve(listener, iconocrust_core::api::router()).await?;
    Ok(())
}

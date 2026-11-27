use iconocrust_core::config::Config;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Load and validate configuration
    let config = Config::from_env().map_err(|e| {
        eprintln!("Configuration error: {}", e);
        e
    })?;

    // 2. Initialize structured logging
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();

    tracing::info!(
        service = "iconocrust-core",
        version = env!("CARGO_PKG_VERSION"),
        host = %config.host,
        port = %config.port,
        "starting Iconocrust Core"
    );

    if !config.is_loopback() {
        tracing::warn!("binding to non-loopback address; ensure auth and TLS are configured");
    }

    // 3. Bind listener
    let address = config.socket_addr();
    let listener = tokio::net::TcpListener::bind(address).await?;
    tracing::info!(%address, "Iconocrust Core is listening");

    // 4. Serve with graceful shutdown
    axum::serve(listener, iconocrust_core::api::router())
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    tracing::info!("Iconocrust Core shut down gracefully");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => { tracing::info!("received Ctrl+C, shutting down"); },
        _ = terminate => { tracing::info!("received SIGTERM, shutting down"); },
    }
}

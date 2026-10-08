use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use minimal_api::config::AppConfig;
use minimal_api::router;
use minimal_api::state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "minimal_api=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration (fails on malformed values, e.g. a non-numeric PORT)
    let config = AppConfig::from_env()?;

    tracing::info!(
        host = %config.host,
        port = config.port,
        "Starting server"
    );

    // Build application state
    let state = AppState::new();

    // Build router
    let app = router::build_router(state);

    // Start server
    let listener =
        tokio::net::TcpListener::bind(format!("{}:{}", config.host, config.port)).await?;
    tracing::info!("Server listening on {}", listener.local_addr()?);

    axum::serve(listener, app).await?;

    Ok(())
}

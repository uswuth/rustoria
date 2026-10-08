use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use modular_monolith::common::config::AppConfig;
use modular_monolith::modules;
use modular_monolith::modules::order::OrderModule;
use modular_monolith::modules::user::UserModule;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "modular_monolith=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = AppConfig::from_env()?;
    tracing::info!(
        host = %config.host,
        port = config.port,
        "Starting modular monolith server"
    );

    // Initialize modules. The order module receives a handle to the user
    // module's service so it can validate user references — the only
    // cross-module dependency, and it goes through the user module's
    // public facade, not its internals.
    let user_module = UserModule::new();
    let order_module = OrderModule::new(user_module.service());

    // Build router
    let app = modules::build_router(user_module, order_module);

    // Start server
    let listener =
        tokio::net::TcpListener::bind(format!("{}:{}", config.host, config.port)).await?;
    tracing::info!("Server listening on {}", listener.local_addr()?);

    axum::serve(listener, app).await?;

    Ok(())
}

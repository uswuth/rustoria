use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use async_service::runtime::Runtime;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "async_service=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting async service");

    // Create and run the runtime
    let runtime = Runtime::new();
    runtime.run().await?;

    tracing::info!("Async service shut down gracefully");
    Ok(())
}

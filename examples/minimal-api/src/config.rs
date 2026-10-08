use std::env;

use anyhow::{anyhow, Context};

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
}

impl AppConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();

        let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());

        // A malformed PORT is a configuration error, not something to
        // silently ignore: fail loudly instead of binding an unexpected port.
        let port = match env::var("PORT") {
            Ok(raw) => raw
                .parse::<u16>()
                .map_err(|e| anyhow!("Invalid PORT value {raw:?}: {e}"))
                .context("PORT must be a number between 0 and 65535")?,
            Err(env::VarError::NotPresent) => 3000,
            Err(e) => return Err(e).context("Failed to read PORT"),
        };

        Ok(Self { host, port })
    }
}

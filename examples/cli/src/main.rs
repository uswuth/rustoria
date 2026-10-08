use clap::Parser;

mod cli;
mod commands;
mod models;
mod storage;

use crate::cli::{Cli, Commands};

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "cli=info".into()),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Add {
            name,
            priority,
            tags,
        } => {
            commands::add::execute(name, priority, tags)?;
        }
        Commands::List { status, limit } => {
            commands::list::execute(status, limit)?;
        }
        Commands::Remove { id } => {
            commands::remove::execute(id)?;
        }
    }

    Ok(())
}

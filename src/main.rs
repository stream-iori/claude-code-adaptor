mod adapters;
mod cli;
mod config;
mod models;
mod services;

use clap::Parser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_target(true)
        .init();

    let cli = cli::Cli::parse();
    cli.run().await
}

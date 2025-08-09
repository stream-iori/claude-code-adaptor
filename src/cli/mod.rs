use clap::{Parser, Subcommand};
use crate::config::Config;
use crate::services::proxy_service::{create_router, AppState};
use axum::serve;
use std::net::SocketAddr;
use tracing::{info, error};

#[derive(Parser)]
#[command(name = "claude-code-adaptor")]
#[command(about = "A proxy adaptor for Claude Code to use Qwen3 API")]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start the proxy server
    Start {
        #[arg(short, long, default_value = "config.json")]
        config_path: String,
    },
    
    /// Check proxy health status
    Health {
        #[arg(short, long, default_value = "http://127.0.0.1:9000")]
        url: String,
    },
}

impl Cli {
    pub async fn run(&self) -> anyhow::Result<()> {
        match &self.command {
            Commands::Start { config_path } => {
                self.start_server(config_path).await
            }
            Commands::Health { url } => {
                self.check_health(url).await
            }
        }
    }
    
    async fn start_server(&self, config_path: &str) -> anyhow::Result<()> {
        let config = Config::load(config_path)?;
        let state = AppState::new(config.clone());

        let server_config = &config.server;
        let addr = SocketAddr::new(server_config.host.parse()?, server_config.port);
        let app = create_router(state);
        
        info!("Starting Claude Code Adaptor server on {}", addr);
        info!("Health check: http://{}/health", addr);

        let listener = tokio::net::TcpListener::bind(&addr).await?;
        serve(listener, app).await?;
            
        Ok(())
    }
    
    async fn check_health(&self, url: &str) -> anyhow::Result<()> {
        let health_url = format!("{}/health", url);
        
        match reqwest::get(&health_url).await {
            Ok(response) if response.status().is_success() => {
                info!("✅ Proxy is healthy at {}", health_url);
                Ok(())
            }
            Ok(response) => {
                error!("❌ Proxy health check failed with status: {}", response.status());
                Err(anyhow::anyhow!("Health check failed"))
            }
            Err(e) => {
                error!("❌ Failed to connect to proxy: {}", e);
                Err(anyhow::anyhow!("Connection failed: {}", e))
            }
        }
    }
}
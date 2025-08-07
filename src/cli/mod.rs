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
        #[arg(short, long, default_value = "127.0.0.1")]
        host: String,
        
        #[arg(short, long, default_value = "8080")]
        port: u16,
    },
    
    /// Check proxy health status
    Health {
        #[arg(short, long, default_value = "http://127.0.0.1:8080")]
        url: String,
    },
    
    /// Display configuration
    Config,
}

impl Cli {
    pub async fn run(&self) -> anyhow::Result<()> {
        match &self.command {
            Commands::Start { host, port } => {
                self.start_server(host, *port).await
            }
            Commands::Health { url } => {
                self.check_health(url).await
            }
            Commands::Config => {
                self.display_config()
            }
        }
    }
    
    async fn start_server(&self, host: &str, port: u16) -> anyhow::Result<()> {
        tracing_subscriber::fmt::init();
        
        let config = Config::load()?;
        let state = AppState::new(config);
        
        let addr = SocketAddr::new(host.parse()?, port);
        let app = create_router(state);
        
        info!("Starting Claude Code Adaptor server on {}", addr);
        info!("Health check: http://{}/health", addr);
        info!("Message: POST http://{}/v1/message", addr);
        
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
    
    fn display_config(&self) -> anyhow::Result<()> {
        let config = Config::load()?;
        
        println!("📋 Configuration:");
        println!("   Server: {}:{}", config.server.host, config.server.port);
        println!("   Qwen Model: {}", config.qwen.model);
        println!("   Qwen Base URL: {}", config.qwen.base_url);
        println!("   Claude Base URL: {}", config.claude.base_url);
        
        Ok(())
    }
}
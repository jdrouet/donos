mod common;
mod dns;

mod config;
mod repository;
mod service;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

fn init_logs() {
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;
    use tracing_subscriber::{fmt, registry, EnvFilter};

    let _ = registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            format!("{}=debug,tower_http=debug", env!("CARGO_PKG_NAME")).into()
        }))
        .with(fmt::layer().with_ansi(cfg!(debug_assertions)))
        .try_init();
}

/// DNS server that filters domain names according to blocklists
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Args {
    /// Path to the configuration file
    #[arg(
        short,
        long,
        default_value = "/etc/donos/donos.toml",
        env = "CONFIG_PATH"
    )]
    config_path: PathBuf,
    #[command(subcommand)]
    inner: Commands,
}

impl Args {
    pub async fn run(self) {
        let config = crate::config::Config::load(&self.config_path);
        match self.inner {
            // Commands::Blocklist(inner) => inner.run(config).await,
            Commands::Dns(inner) => inner.run(config).await,
        }
    }
}

#[derive(Debug, Subcommand)]
enum Commands {
    // Blocklist(crate::cmd::blocklist::Command),
    Dns(crate::dns::Command),
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    init_logs();

    Args::parse().run().await;

    Ok(())
}

pub mod blocklist;
pub mod dns;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

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
            Commands::Blocklist(inner) => inner.run(config).await,
            Commands::Dns(inner) => inner.run(config).await,
        }
    }
}

#[derive(Debug, Subcommand)]
enum Commands {
    Blocklist(blocklist::Command),
    Dns(dns::Command),
}

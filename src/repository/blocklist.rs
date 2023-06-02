use donos_blocklist_loader::BlocklistKind;
use sqlx::{Pool, Sqlite};
use std::{collections::BTreeMap, error::Error, net::SocketAddr};

#[derive(Debug, serde::Deserialize)]
pub struct BlocklistItem {
    pub url: String,
    pub kind: BlocklistKind,
}

#[derive(Debug, Default, serde::Deserialize)]
pub struct Config {
    #[serde(flatten)]
    pub inner: BTreeMap<String, BlocklistItem>,
}

impl Config {
    pub fn build(&self, database: Pool<Sqlite>) -> DatabaseBlocklistService {
        DatabaseBlocklistService::new(database)
    }
}

#[async_trait::async_trait]
pub trait BlocklistService {
    async fn is_blocked(&self, origin: &SocketAddr, domain: &str) -> Result<bool, Box<dyn Error>>;
}

#[derive(Debug, Clone)]
pub struct DatabaseBlocklistService {
    #[allow(dead_code)]
    database: Pool<Sqlite>,
}

impl DatabaseBlocklistService {
    pub fn new(database: Pool<Sqlite>) -> Self {
        Self { database }
    }
}

#[async_trait::async_trait]
impl BlocklistService for DatabaseBlocklistService {
    #[tracing::instrument(skip(self, _origin))]
    async fn is_blocked(&self, _origin: &SocketAddr, domain: &str) -> Result<bool, Box<dyn Error>> {
        tracing::debug!("checking in the blocklist");
        let exists: bool =
            sqlx::query_scalar("SELECT count(id) > 0 FROM blocked_domains WHERE domain = ?")
                .bind(domain)
                .fetch_one(&self.database)
                .await?;
        Ok(exists)
    }
}

#[derive(Debug, Default)]
pub struct MemoryBlocklistService {
    inner: std::collections::HashSet<String>,
}

#[cfg(test)]
impl MemoryBlocklistService {
    pub fn with_domain<D: Into<String>>(mut self, domain: D) -> Self {
        self.inner.insert(domain.into());
        self
    }
}

#[async_trait::async_trait]
impl BlocklistService for MemoryBlocklistService {
    #[tracing::instrument(skip(self, _origin))]
    async fn is_blocked(&self, _origin: &SocketAddr, domain: &str) -> Result<bool, Box<dyn Error>> {
        tracing::debug!("checking in the blocklist");
        Ok(self.inner.contains(domain))
    }
}

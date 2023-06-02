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

#[async_trait::async_trait]
pub trait BlocklistService {
    async fn is_blocked(&self, origin: &SocketAddr, domain: &str) -> Result<bool, Box<dyn Error>>;
}

#[derive(Debug)]
pub struct DatabaseBlocklistService {
    inner: Pool<Sqlite>,
}

impl DatabaseBlocklistService {
    pub async fn new(url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let pool = sqlx::SqlitePool::connect(url).await?;

        Ok(Self { inner: pool })
    }
}

#[async_trait::async_trait]
impl BlocklistService for DatabaseBlocklistService {
    #[tracing::instrument(skip(self, _origin))]
    async fn is_blocked(&self, _origin: &SocketAddr, domain: &str) -> Result<bool, Box<dyn Error>> {
        tracing::debug!("checking in the blocklist");
        Ok(false)
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

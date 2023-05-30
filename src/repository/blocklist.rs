use donos_blocklist_loader::BlocklistKind;
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

#[derive(Debug, Default)]
pub struct MockBlocklistService {
    inner: std::collections::HashSet<&'static str>,
}

#[cfg(test)]
impl MockBlocklistService {
    pub fn with_domain(mut self, domain: &'static str) -> Self {
        self.inner.insert(domain);
        self
    }
}

#[async_trait::async_trait]
impl BlocklistService for MockBlocklistService {
    #[tracing::instrument(skip(self, _origin))]
    async fn is_blocked(&self, _origin: &SocketAddr, domain: &str) -> Result<bool, Box<dyn Error>> {
        tracing::debug!("checking in the blocklist");
        Ok(self.inner.contains(domain))
    }
}

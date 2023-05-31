use donos_proto::packet::record::Record;
use donos_proto::packet::QueryType;
use std::io::Result;

#[derive(Debug, Default, serde::Deserialize)]
pub struct Config {}

impl Config {
    pub async fn build(self) -> Result<RemoteCacheService> {
        RemoteCacheService::new(self).await
    }
}

#[async_trait::async_trait]
pub trait CacheService {
    async fn request(&self, qname: &str, qtype: QueryType) -> Result<Option<Vec<Record>>>;
}

pub struct RemoteCacheService {}

impl RemoteCacheService {
    async fn new(_config: Config) -> Result<Self> {
        Ok(Self {})
    }
}

#[async_trait::async_trait]
impl CacheService for RemoteCacheService {
    #[tracing::instrument(skip(self))]
    async fn request(&self, qname: &str, qtype: QueryType) -> Result<Option<Vec<Record>>> {
        Ok(None)
    }
}

// #[cfg(test)]
#[derive(Debug, Default)]
pub struct MockCacheService {
    inner: std::collections::HashMap<(&'static str, QueryType), Vec<Record>>,
}

// #[cfg(test)]
impl MockCacheService {
    pub fn with_records(
        mut self,
        address: &'static str,
        qtype: QueryType,
        records: Vec<Record>,
    ) -> Self {
        self.inner.insert((address, qtype), records);
        self
    }
}

// #[cfg(test)]
#[async_trait::async_trait]
impl CacheService for MockCacheService {
    async fn request(&self, qname: &str, qtype: QueryType) -> Result<Option<Vec<Record>>> {
        if let Some(found) = self.inner.get(&(qname, qtype)) {
            Ok(Some(found.clone()))
        } else {
            Ok(None)
        }
    }
}

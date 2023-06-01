use donos_proto::packet::record::Record;
use donos_proto::packet::QueryType;
use moka::future::Cache;
use std::io::Result;
use std::ops::Add;
use std::time::{Duration, SystemTime};

#[derive(Debug, serde::Deserialize)]
pub struct Config {
    #[serde(default = "Config::default_size")]
    size: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self { size: 1000 }
    }
}

impl Config {
    pub fn default_size() -> u64 {
        1000
    }
}

impl Config {
    pub async fn build(self) -> Result<RemoteCacheService> {
        RemoteCacheService::new(self).await
    }
}

#[async_trait::async_trait]
pub trait CacheService {
    async fn persist(&self, qname: &str, qtype: QueryType, records: Vec<Record>) -> Result<()>;
    async fn request(&self, qname: &str, qtype: QueryType) -> Result<Option<Vec<Record>>>;
}

pub struct RemoteCacheService {
    inner: Cache<(String, QueryType), (SystemTime, Vec<Record>)>,
}

impl RemoteCacheService {
    async fn new(config: Config) -> Result<Self> {
        Ok(Self {
            inner: Cache::new(config.size),
        })
    }
}

#[async_trait::async_trait]
impl CacheService for RemoteCacheService {
    #[tracing::instrument(skip(self, records))]
    async fn persist(&self, qname: &str, qtype: QueryType, records: Vec<Record>) -> Result<()> {
        if let Some(min_ttl) = records.iter().map(|item| item.ttl()).min() {
            tracing::debug!("persisting with a ttl of {min_ttl} seconds");
            let deadline = SystemTime::now().add(Duration::new(min_ttl as u64, 0));
            self.inner
                .insert((qname.to_string(), qtype), (deadline, records))
                .await;
        }
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn request(&self, qname: &str, qtype: QueryType) -> Result<Option<Vec<Record>>> {
        let key = (qname.to_string(), qtype);
        if let Some((until, records)) = self.inner.get(&key) {
            let now = SystemTime::now();
            if let Ok(diff) = until.duration_since(now) {
                tracing::debug!("found in cache with a ttl of {} seconds", diff.as_secs());
                Ok(Some(
                    records
                        .iter()
                        .map(|record| record.delayed_ttl(diff.as_secs() as u32))
                        .collect(),
                ))
            } else {
                tracing::debug!("found in cache but expired");
                self.inner.invalidate(&key).await;
                Ok(None)
            }
        } else {
            tracing::debug!("not found in cache");
            Ok(None)
        }
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
    async fn persist(&self, _qname: &str, _qtype: QueryType, _records: Vec<Record>) -> Result<()> {
        Ok(())
    }

    async fn request(&self, qname: &str, qtype: QueryType) -> Result<Option<Vec<Record>>> {
        if let Some(found) = self.inner.get(&(qname, qtype)) {
            Ok(Some(found.clone()))
        } else {
            Ok(None)
        }
    }
}

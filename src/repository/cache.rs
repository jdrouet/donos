use donos_parser::packet::record::Record;
use donos_parser::packet::QueryType;
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
    pub async fn build(self) -> Result<MemoryCacheService> {
        Ok(MemoryCacheService::new(self.size))
    }
}

#[async_trait::async_trait]
pub trait CacheService {
    async fn persist(&self, qname: &str, qtype: QueryType, records: Vec<Record>) -> Result<()>;
    async fn request(&self, qname: &str, qtype: QueryType) -> Result<Option<Vec<Record>>>;
}

pub struct MemoryCacheService {
    inner: Cache<(String, QueryType), (SystemTime, Vec<Record>)>,
}

impl MemoryCacheService {
    #[inline]
    fn new(size: u64) -> Self {
        Self {
            inner: Cache::new(size),
        }
    }
}

#[async_trait::async_trait]
impl CacheService for MemoryCacheService {
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

#[cfg(test)]
#[derive(Debug, Default)]
pub struct MockCacheService {
    inner: std::collections::HashMap<(&'static str, QueryType), Vec<Record>>,
}

#[cfg(test)]
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

#[cfg(test)]
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

#[cfg(test)]
mod tests {
    use std::{
        net::Ipv4Addr,
        ops::{Add, Sub},
        time::{Duration, SystemTime},
    };

    use super::{CacheService, MemoryCacheService};
    use donos_parser::packet::{record::Record, QueryType};

    #[tokio::test]
    async fn should_persist_in_cache() {
        let srv = MemoryCacheService::new(10);
        srv.persist(
            "perdu.com",
            QueryType::A,
            vec![Record::A {
                domain: "perdu.com".into(),
                addr: Ipv4Addr::new(1, 2, 3, 4),
                ttl: 60,
            }],
        )
        .await
        .unwrap();
        let found = srv.inner.get(&("perdu.com".to_string(), QueryType::A));
        assert!(found.is_some());
    }

    #[tokio::test]
    async fn should_not_return_if_outdated() {
        let srv = MemoryCacheService::new(10);
        srv.inner
            .insert(
                ("perdu.com".to_string(), QueryType::A),
                (
                    SystemTime::now().sub(Duration::new(10, 0)),
                    vec![Record::A {
                        domain: "perdu.com".into(),
                        addr: Ipv4Addr::new(1, 2, 3, 4),
                        ttl: 5,
                    }],
                ),
            )
            .await;
        let found = srv.request("perdu.com", QueryType::A).await.unwrap();
        assert!(found.is_none());
        // should flush
        assert!(srv
            .inner
            .get(&("perdu.com".to_string(), QueryType::A))
            .is_none());
    }

    #[tokio::test]
    async fn should_return() {
        let srv = MemoryCacheService::new(10);
        srv.inner
            .insert(
                ("perdu.com".to_string(), QueryType::A),
                (
                    SystemTime::now().add(Duration::new(60, 0)),
                    vec![Record::A {
                        domain: "perdu.com".into(),
                        addr: Ipv4Addr::new(1, 2, 3, 4),
                        ttl: 180,
                    }],
                ),
            )
            .await;
        let found = srv
            .request("perdu.com", QueryType::A)
            .await
            .unwrap()
            .unwrap();
        for item in found {
            assert_eq!(item.ttl(), 59);
        }
    }
}

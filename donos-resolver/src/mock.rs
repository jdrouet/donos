use std::collections::HashMap;

use crate::prelude::{Resolver, ResolverError};
use donos_parser::packet::{DnsPacket, QueryType};

#[derive(Debug)]
pub struct MockResolver {
    identifier: String,
    responses: HashMap<(QueryType, &'static str), DnsPacket>,
}

impl MockResolver {
    pub fn new<I: Into<String>>(identifier: I) -> Self {
        Self {
            identifier: identifier.into(),
            responses: Default::default(),
        }
    }
}

#[async_trait::async_trait]
impl Resolver for MockResolver {
    fn kind(&self) -> &'static str {
        "mock-resolver"
    }

    fn identifier(&self) -> &str {
        &self.identifier
    }

    async fn resolve(&self, kind: QueryType, hostname: &str) -> Result<DnsPacket, ResolverError> {
        if let Some(found) = self.responses.get(&(kind, hostname)) {
            Ok(found.clone())
        } else {
            Err(ResolverError::Unknown)
        }
    }
}

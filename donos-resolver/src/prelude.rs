use donos_parser::packet::{DnsPacket, QueryType};

#[derive(Clone, Debug)]
pub enum ResolverError {
    Unknown,
}

#[async_trait::async_trait]
pub trait Resolver: std::fmt::Debug {
    fn kind(&self) -> &'static str;
    fn identifier(&self) -> &str;

    async fn resolve(&self, kind: QueryType, hostname: &str) -> Result<DnsPacket, ResolverError>;
}

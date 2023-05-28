use donos_proto::buffer::BytePacketBuffer;
use donos_proto::packet::question::Question;
use donos_proto::packet::{DnsPacket, QueryType};
use std::io::Result;
use std::sync::atomic::{AtomicU16, Ordering};
use tokio::net::UdpSocket;

#[derive(Debug, serde::Deserialize)]
pub struct Config {
    #[serde(default = "Config::default_servers")]
    pub servers: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            servers: Self::default_servers(),
        }
    }
}

impl Config {
    pub fn default_servers() -> Vec<String> {
        vec!["1.1.1.1".to_string(), "1.0.0.1".to_string()]
    }
}

impl Config {
    pub async fn build(self) -> Result<RemoteLookupService> {
        RemoteLookupService::new(self).await
    }
}

#[async_trait::async_trait]
pub trait LookupService {
    async fn lookup(&self, qname: &str, qtype: QueryType) -> Result<DnsPacket>;
}

pub struct RemoteLookupService {
    socket: UdpSocket,
    servers: Vec<(String, u16)>,
    index: AtomicU16,
}

impl RemoteLookupService {
    async fn new(config: Config) -> Result<Self> {
        let socket = UdpSocket::bind(("0.0.0.0", 43210)).await?;

        Ok(Self {
            socket,
            servers: config.servers.into_iter().map(|item| (item, 53)).collect(),
            index: AtomicU16::new(0),
        })
    }
}

#[async_trait::async_trait]
impl LookupService for RemoteLookupService {
    async fn lookup(&self, qname: &str, qtype: QueryType) -> Result<DnsPacket> {
        let mut packet = DnsPacket::default();

        packet.header.inner.id = self.index.fetch_add(1, Ordering::SeqCst);
        packet.header.questions = 1;
        packet.header.inner.recursion_desired = true;
        packet
            .questions
            .push(Question::new(qname.to_string(), qtype));

        let req_buffer = packet.create_buffer()?;
        self.socket
            .send_to(&req_buffer.buf[0..req_buffer.pos], &self.servers[0])
            .await?;

        let mut res_buffer = BytePacketBuffer::default();
        self.socket.recv_from(&mut res_buffer.buf).await?;

        Ok(DnsPacket::try_from(res_buffer)?)
    }
}

#[cfg(test)]
#[derive(Debug, Default)]
pub struct MockLookupService {
    inner: std::collections::HashMap<(&'static str, QueryType), DnsPacket>,
}

#[cfg(test)]
impl MockLookupService {
    pub fn with_query(
        mut self,
        address: &'static str,
        qtype: QueryType,
        packet: DnsPacket,
    ) -> Self {
        self.inner.insert((address, qtype), packet);
        self
    }
}

#[cfg(test)]
#[async_trait::async_trait]
impl LookupService for MockLookupService {
    async fn lookup(&self, qname: &str, qtype: QueryType) -> Result<DnsPacket> {
        if let Some(found) = self.inner.get(&(qname, qtype)) {
            Ok(found.clone())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "network issue",
            ))
        }
    }
}

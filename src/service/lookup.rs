use donos_parser::{BytePacketBuffer, DnsPacket, DnsQuestion, QueryType};
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
    pub async fn build(self) -> Result<LookupService> {
        LookupService::new(self).await
    }
}

pub struct LookupService {
    socket: UdpSocket,
    servers: Vec<(String, u16)>,
    index: AtomicU16,
}

impl LookupService {
    async fn new(config: Config) -> Result<Self> {
        let socket = UdpSocket::bind(("0.0.0.0", 43210)).await?;

        Ok(Self {
            socket,
            servers: config.servers.into_iter().map(|item| (item, 53)).collect(),
            index: AtomicU16::new(0),
        })
    }

    pub async fn execute(&self, qname: &str, qtype: QueryType) -> Result<DnsPacket> {
        let mut packet = DnsPacket::default();

        packet.header.id = self.index.fetch_add(1, Ordering::SeqCst);
        packet.header.questions = 1;
        packet.header.recursion_desired = true;
        packet
            .questions
            .push(DnsQuestion::new(qname.to_string(), qtype));

        let req_buffer = packet.create_buffer()?;
        self.socket
            .send_to(&req_buffer.buf[0..req_buffer.pos], &self.servers[0])
            .await?;

        let mut res_buffer = BytePacketBuffer::default();
        self.socket.recv_from(&mut res_buffer.buf).await?;

        Ok(DnsPacket::try_from(res_buffer)?)
    }
}

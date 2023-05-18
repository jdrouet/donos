use donos_parser::{BytePacketBuffer, DnsPacket, DnsQuestion, QueryType};
use std::io::Result;
use std::sync::atomic::{AtomicU16, Ordering};
use tokio::net::UdpSocket;

pub struct LookupService {
    socket: UdpSocket,
    server: (&'static str, u16),
    index: AtomicU16,
}

impl LookupService {
    pub async fn new() -> Result<Self> {
        let socket = UdpSocket::bind(("0.0.0.0", 43210)).await?;
        let server = ("1.1.1.1", 53);

        Ok(Self {
            socket,
            server,
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
            .send_to(&req_buffer.buf[0..req_buffer.pos], self.server)
            .await?;

        let mut res_buffer = BytePacketBuffer::default();
        self.socket.recv_from(&mut res_buffer.buf).await?;

        Ok(DnsPacket::try_from(res_buffer)?)
    }
}

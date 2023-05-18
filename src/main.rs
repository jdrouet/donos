use buffer::{ReaderError, WriterError};
use tokio::net::UdpSocket;

mod buffer;
mod packet;
mod repository;
mod service;

use crate::buffer::BytePacketBuffer;
use crate::packet::{DnsPacket, ResponseCode};
use crate::service::database::{Config as DatabaseConfig, Error as DatabaseError, Pool};
use crate::service::lookup::LookupService;

fn init_logs() {
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;
    use tracing_subscriber::{fmt, registry, EnvFilter};

    let _ = registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            format!("{}=debug,tower_http=debug", env!("CARGO_PKG_NAME")).into()
        }))
        .with(fmt::layer().with_ansi(cfg!(debug_assertions)))
        .try_init();
}

#[derive(Debug)]
pub enum HandleError {
    Database(DatabaseError),
    Writer(WriterError),
    Reader(ReaderError),
    Io(std::io::Error),
}

impl From<DatabaseError> for HandleError {
    fn from(value: DatabaseError) -> Self {
        Self::Database(value)
    }
}

impl From<WriterError> for HandleError {
    fn from(value: WriterError) -> Self {
        Self::Writer(value)
    }
}

impl From<ReaderError> for HandleError {
    fn from(value: ReaderError) -> Self {
        Self::Reader(value)
    }
}

impl From<std::io::Error> for HandleError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

pub struct DnsServer {
    database: Pool,
    lookup: LookupService,
    socket: UdpSocket,
}

impl DnsServer {
    pub async fn new(address: &str) -> Result<Self, HandleError> {
        tracing::info!("starting dns server");
        let database = DatabaseConfig::from_env().build().await?;
        let lookup = LookupService::new().await?;
        let socket = UdpSocket::bind(address).await?;

        Ok(Self {
            database,
            lookup,
            socket,
        })
    }

    async fn handle(&self) -> Result<(), HandleError> {
        // With a socket ready, we can go ahead and read a packet. This will
        // block until one is received.
        let mut req_buffer = BytePacketBuffer::new();

        // The `recv_from` function will write the data into the provided buffer,
        // and return the length of the data read as well as the source address.
        // We're not interested in the length, but we need to keep track of the
        // source in order to send our reply later on.
        let (_, src) = self.socket.recv_from(&mut req_buffer.buf).await?;
        tracing::debug!("received from {:?}", src.ip());

        // Next, `DnsPacket::from_buffer` is used to parse the raw bytes into
        // a `DnsPacket`.
        let mut request = DnsPacket::from_buffer(&mut req_buffer)?;

        let mut tx = self.database.begin().await?;

        // Create and initialize the response packet
        let mut packet = DnsPacket::new();
        packet.header.id = request.header.id;
        packet.header.recursion_desired = true;
        packet.header.recursion_available = true;
        packet.header.response = true;

        // In the normal case, exactly one question is present
        if let Some(question) = request.questions.pop() {
            tracing::debug!("query: {question:?}");

            if repository::host::is_blocked(&mut tx, &src, &question.name).await? {
                tracing::error!("qname {} is blocked for {src:?}", question.name);
                packet.header.response_code = ResponseCode::ServerFailure;
            } else {
                // Since all is set up and as expected, the query can be forwarded to the
                // target server. There's always the possibility that the query will
                // fail, in which case the `SERVFAIL` response code is set to indicate
                // as much to the client. If rather everything goes as planned, the
                // question and response records as copied into our response packet.
                match self.lookup.execute(&question.name, question.qtype).await {
                    Ok(result) => {
                        packet.questions.push(question);
                        packet.header.response_code = result.header.response_code;

                        for rec in result.answers {
                            tracing::debug!("answer: {rec:?}");
                            packet.answers.push(rec);
                        }
                        for rec in result.authorities {
                            tracing::debug!("authority: {rec:?}");
                            packet.authorities.push(rec);
                        }
                        for rec in result.resources {
                            tracing::debug!("resource: {rec:?}");
                            packet.resources.push(rec);
                        }
                    }
                    Err(error) => {
                        tracing::error!("unable to lookup question: {error:?}");
                        packet.header.response_code = ResponseCode::ServerFailure;
                    }
                }
            }
        }
        // Being mindful of how unreliable input data from arbitrary senders can be, we
        // need make sure that a question is actually present. If not, we return `FORMERR`
        // to indicate that the sender made something wrong.
        else {
            packet.header.response_code = ResponseCode::FormatError;
        }

        // The only thing remaining is to encode our response and send it off!
        let mut res_buffer = BytePacketBuffer::new();
        packet.write(&mut res_buffer)?;

        let len = res_buffer.pos();
        let data = res_buffer.get_range(0, len)?;

        self.socket.send_to(data, src).await?;

        Ok(())
    }

    pub async fn run(&self) {
        tracing::info!("running dns server");
        loop {
            match self.handle().await {
                Ok(_) => {}
                Err(err) => tracing::error!("an error occured: {err:?}"),
            }
        }
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    init_logs();

    let dns_server = DnsServer::new("0.0.0.0:2053")
        .await
        .expect("unable to create dns server");
    dns_server.run().await;

    Ok(())
}

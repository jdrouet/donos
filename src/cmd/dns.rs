use crate::service::database::{Error as DatabaseError, Pool};
use crate::service::lookup::LookupService;
use clap::Args;
use donos_parser::{BytePacketBuffer, DnsPacket, ReaderError, ResponseCode, WriterError};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::net::UdpSocket;

#[derive(Debug, serde::Deserialize)]
pub struct Config {
    #[serde(default = "Config::default_host")]
    pub host: IpAddr,
    #[serde(default = "Config::default_port")]
    pub port: u16,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: Self::default_host(),
            port: Self::default_port(),
        }
    }
}

impl Config {
    fn default_host() -> IpAddr {
        IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))
    }

    fn default_port() -> u16 {
        53
    }
}

impl Config {
    pub fn address(&self) -> SocketAddr {
        SocketAddr::from((self.host, self.port))
    }
}

/// Starts the DNS server, the core of the machine
#[derive(Args, Debug)]
pub struct Command;

impl Command {
    pub async fn run(&self, config: crate::config::Config) {
        let dns_server = DnsServer::new(config)
            .await
            .expect("unable to create dns server");
        dns_server.run().await;
    }
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
    pub async fn new(config: crate::config::Config) -> Result<Self, HandleError> {
        tracing::info!("preparing dns server");
        let database = config.database.build().await?;
        crate::service::database::migrate(&database).await?;
        let lookup = config.lookup.build().await?;

        let address = config.dns.address();
        tracing::info!("starting dns server on {address:?}");
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
        let mut req_buffer = BytePacketBuffer::default();

        // The `recv_from` function will write the data into the provided buffer,
        // and return the length of the data read as well as the source address.
        // We're not interested in the length, but we need to keep track of the
        // source in order to send our reply later on.
        let (_, src) = self.socket.recv_from(&mut req_buffer.buf).await?;
        tracing::debug!("received from {:?}", src.ip());

        // Next, `DnsPacket::from_buffer` is used to parse the raw bytes into
        // a `DnsPacket`.
        let mut request = DnsPacket::try_from(req_buffer)?;

        let mut tx = self.database.begin().await?;

        // Create and initialize the response packet
        let mut packet = DnsPacket::default();
        packet.header.id = request.header.id;
        packet.header.recursion_desired = true;
        packet.header.recursion_available = true;
        packet.header.response = true;

        // In the normal case, exactly one question is present
        if let Some(question) = request.questions.pop() {
            tracing::debug!("query: {question:?}");

            if crate::model::blocklist::is_blocked(&mut tx, &src.ip(), &question.name).await? {
                tracing::error!("qname {} is blocked for {src:?}", question.name);
                packet.header.response_code = ResponseCode::NameError;
            } else if let Some(found) =
                crate::model::record::find(&mut tx, question.qtype, &question.name).await?
            {
                tracing::debug!("{:?} {} found in cache", question.qtype, question.name);
                packet.questions.push(question);
                packet.header.response_code = ResponseCode::NoError;

                packet.answers.push(found);
            } else {
                tracing::debug!(
                    "{:?} {} not found in cache, resolving",
                    question.qtype,
                    question.name
                );
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

                            match crate::model::record::persist(&mut tx, &rec).await {
                                Ok(_) => tracing::debug!("persisted in cache"),
                                Err(err) => tracing::error!("unable to persist in cache: {err:?}"),
                            };

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
        let res_buffer = packet.create_buffer()?;

        let len = res_buffer.pos();
        let data = res_buffer.get_range(0, len)?;

        self.socket.send_to(data, src).await?;

        match tx.commit().await {
            Ok(_) => {}
            Err(err) => tracing::error!("unable to commit transaction: {err:?}"),
        };

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

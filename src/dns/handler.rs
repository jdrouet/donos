use super::error::HandleError;
use crate::repository::blocklist::BlocklistService;
use crate::repository::cache::CacheService;
use crate::repository::lookup::LookupService;
use donos_parser::buffer::BytePacketBuffer;
use donos_parser::packet::header::ResponseCode;
use donos_parser::packet::DnsPacket;
use donos_server::prelude::Message;
use std::net::SocketAddr;
use std::sync::Arc;

#[allow(dead_code)]
pub(crate) struct DnsHandler {
    blocklist: Arc<dyn BlocklistService + Send + Sync>,
    cache: Arc<dyn CacheService + Send + Sync>,
    lookup: Arc<dyn LookupService + Sync + Send>,
}

impl DnsHandler {
    pub fn new(
        blocklist: Arc<dyn BlocklistService + Send + Sync>,
        cache: Arc<dyn CacheService + Send + Sync>,
        lookup: Arc<dyn LookupService + Sync + Send>,
    ) -> Self {
        Self {
            blocklist,
            cache,
            lookup,
        }
    }
}

impl DnsHandler {
    async fn try_handle(
        &self,
        origin: &SocketAddr,
        packet: &DnsPacket,
    ) -> Result<DnsPacket, HandleError> {
        let question = match packet.questions.first() {
            Some(found) => found,
            None => return Err(HandleError::NoQuestion),
        };
        if self
            .blocklist
            .is_blocked(origin, question.name.as_str())
            .await
            .map_err(HandleError::Blocklist)?
        {
            let mut res = DnsPacket::response_from(packet);
            res.header.response_code = ResponseCode::NameError;
            return Ok(res);
        }

        if let Some(records) = self
            .cache
            .request(question.name.as_str(), question.qtype)
            .await
            .map_err(HandleError::Cache)?
        {
            return Ok(DnsPacket::response_from(packet).with_answers(records));
        }

        let response = self
            .lookup
            .lookup(question.name.as_str(), question.qtype)
            .await
            .map_err(HandleError::Lookup)?;

        if let Err(error) = self
            .cache
            .persist(
                question.name.as_str(),
                question.qtype,
                response.answers.clone(),
            )
            .await
        {
            tracing::error!("couldn't persist in cache: {error:?}");
        }

        let res = DnsPacket::response_from(packet).with_answers(response.answers);

        Ok(res)
    }
}

#[async_trait::async_trait]
impl donos_server::Handler for DnsHandler {
    #[tracing::instrument(skip_all, fields(origin = ?message.address, id = tracing::field::Empty))]
    async fn handle(&self, message: Message) -> Option<Message> {
        let Message {
            address,
            buffer,
            size: _,
        } = message;

        // With a socket ready, we can go ahead and read a packet. This will
        // block until one is received.
        let buffer = BytePacketBuffer::new(buffer);
        // Next, `DnsPacket::from_buffer` is used to parse the raw bytes into
        // a `DnsPacket`.
        let request = match DnsPacket::try_from(buffer) {
            Ok(req) => req,
            Err(err) => {
                tracing::debug!("unable to read packet: {err:?}");
                return None;
            }
        };

        tracing::Span::current().record("id", request.header.id);

        match self.try_handle(&address, &request).await {
            Ok(packet) => {
                tracing::debug!("creating response");
                let buffer = packet.create_buffer().unwrap();

                Some(Message {
                    address,
                    buffer: buffer.buf,
                    size: buffer.pos,
                })
            }
            Err(HandleError::NoQuestion) => {
                tracing::debug!("no question where specified");
                None
            }
            Err(error) => {
                tracing::warn!("unable to build response message: {error:?}");

                todo!()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DnsHandler;
    use crate::repository::blocklist::MemoryBlocklistService;
    use crate::repository::cache::MockCacheService;
    use crate::repository::lookup::MockLookupService;
    use donos_parser::buffer::BytePacketBuffer;
    use donos_parser::packet::header::{Header, ResponseCode};
    use donos_parser::packet::question::{DnsClass, Question};
    use donos_parser::packet::record::Record;
    use donos_parser::packet::{DnsPacket, QueryType};
    use donos_server::{prelude::Message, Handler};
    use similar_asserts::assert_eq;
    use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
    use std::sync::Arc;

    fn socket_address() -> SocketAddr {
        SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 1, 0, 1), 42))
    }

    #[tokio::test]
    async fn should_resolve_query() {
        crate::init_logs();

        let input_packet = DnsPacket::new(Header::question(1))
            .with_question(Question::new("perdu.com".into(), QueryType::A));
        let input_buffer = input_packet.create_buffer().unwrap();
        let input = Message {
            address: socket_address(),
            buffer: input_buffer.buf,
            size: input_buffer.pos,
        };

        let blocklist = Arc::new(MemoryBlocklistService::default());
        let cache = Arc::new(MockCacheService::default());
        let lookup = Arc::new(
            MockLookupService::default().with_query(
                "perdu.com",
                QueryType::A,
                DnsPacket::new(Header::response(10))
                    .with_question(Question {
                        name: "perdu.com".into(),
                        qtype: QueryType::A,
                        qclass: DnsClass::Internet,
                    })
                    .with_answer(Record::A {
                        domain: "perdu.com".into(),
                        addr: Ipv4Addr::new(99, 99, 99, 99),
                        ttl: 100,
                    }),
            ),
        );
        let result = DnsHandler::new(blocklist, cache, lookup)
            .handle(input)
            .await;

        let result = result.expect("should have a message");
        let result = BytePacketBuffer::new(result.buffer);
        let result = DnsPacket::try_from(result).unwrap();

        assert_eq!(result.header.id, input_packet.header.id);
    }

    #[tokio::test]
    async fn should_block_query() {
        crate::init_logs();

        let input_packet = DnsPacket::new(Header::question(1))
            .with_question(Question::new("www.facebook.com".into(), QueryType::A));
        let input_buffer = input_packet.create_buffer().unwrap();
        let input = Message {
            address: socket_address(),
            buffer: input_buffer.buf,
            size: input_buffer.pos,
        };

        let blocklist = Arc::new(MemoryBlocklistService::default().with_domain("www.facebook.com"));
        let cache = Arc::new(MockCacheService::default());
        let lookup = Arc::new(
            MockLookupService::default().with_query(
                "www.facebook.com",
                QueryType::A,
                DnsPacket::new(Header::response(10))
                    .with_question(Question {
                        name: "www.facebook.com".into(),
                        qtype: QueryType::A,
                        qclass: DnsClass::Internet,
                    })
                    .with_answer(Record::A {
                        domain: "www.facebook.com".into(),
                        addr: Ipv4Addr::new(99, 99, 99, 99),
                        ttl: 100,
                    }),
            ),
        );
        let result = DnsHandler::new(blocklist, cache, lookup)
            .handle(input)
            .await;

        let result = result.expect("should have a message");
        let result = BytePacketBuffer::new(result.buffer);
        let result = DnsPacket::try_from(result).unwrap();

        assert_eq!(result.header.id, 1);
        assert!(result.header.response);
        assert_eq!(result.header.response_code, ResponseCode::NameError);
    }

    #[tokio::test]
    async fn should_not_answer_if_not_question() {
        crate::init_logs();

        let input_packet = DnsPacket::new(Header::question(1));
        let input_buffer = input_packet.create_buffer().unwrap();
        let input = Message {
            address: socket_address(),
            buffer: input_buffer.buf,
            size: input_buffer.pos,
        };

        let blocklist = Arc::new(MemoryBlocklistService::default());
        let cache = Arc::new(MockCacheService::default());
        let lookup = Arc::new(MockLookupService::default());
        let result = DnsHandler::new(blocklist, cache, lookup)
            .handle(input)
            .await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn should_use_cache() {
        crate::init_logs();

        let input_packet = DnsPacket::new(Header::question(1))
            .with_question(Question::new("perdu.com".into(), QueryType::A));
        let input_buffer = input_packet.create_buffer().unwrap();
        let input = Message {
            address: socket_address(),
            buffer: input_buffer.buf,
            size: input_buffer.pos,
        };

        let blocklist = Arc::new(MemoryBlocklistService::default());
        let cache = Arc::new(MockCacheService::default().with_records(
            "perdu.com",
            QueryType::A,
            vec![Record::A {
                domain: "perdu.com".into(),
                addr: Ipv4Addr::new(10, 0, 0, 1),
                ttl: 42,
            }],
        ));
        let lookup = Arc::new(MockLookupService::default());
        let result = DnsHandler::new(blocklist, cache, lookup)
            .handle(input)
            .await;

        let result = result.expect("should have a message");
        let result = BytePacketBuffer::new(result.buffer);
        let result = DnsPacket::try_from(result).unwrap();

        assert_eq!(result.header.id, 1);
        assert!(result.header.response);
        assert_eq!(result.header.response_code, ResponseCode::NoError);
        assert_eq!(result.answers.len(), 1);
    }
}

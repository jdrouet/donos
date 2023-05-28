use super::error::HandleError;
use crate::service::blocklist::BlocklistService;
use crate::service::lookup::LookupService;
use donos_proto::buffer::BytePacketBuffer;
use donos_proto::packet::header::{Header, PartialHeader, ResponseCode};
use donos_proto::packet::DnsPacket;
use donos_server::prelude::Message;
use std::net::SocketAddr;
use std::sync::Arc;

#[allow(dead_code)]
pub(crate) struct DnsHandler {
    blocklist: Arc<dyn BlocklistService + Send + Sync>,
    lookup: Arc<dyn LookupService + Sync + Send>,
}

impl DnsHandler {
    pub fn new(
        blocklist: Arc<dyn BlocklistService + Send + Sync>,
        lookup: Arc<dyn LookupService + Sync + Send>,
    ) -> Self {
        Self { blocklist, lookup }
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
            return Err(HandleError::Blocked);
        }

        let mut result = self
            .lookup
            .lookup(question.name.as_str(), question.qtype)
            .await
            .map_err(|err| HandleError::Lookup(err))?;

        result.header.inner.id = packet.header.inner.id;

        Ok(result)
    }
}

#[async_trait::async_trait]
impl donos_server::Handler for DnsHandler {
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

        match self.try_handle(&address, &request).await {
            Ok(packet) => {
                let buffer = packet.create_buffer().unwrap();

                Some(Message {
                    address,
                    buffer: buffer.buf,
                    size: buffer.pos,
                })
            }
            Err(HandleError::Blocked) => {
                let mut result = request;
                result.header.inner.response = true;
                result.header.inner.response_code = ResponseCode::NameError;
                let buffer = result.create_buffer().unwrap();

                Some(Message {
                    address,
                    buffer: buffer.buf,
                    size: buffer.pos,
                })
            }
            Err(error) => {
                eprintln!("received error {error:?}");
                tracing::warn!("unable to build response message: {error:?}");

                todo!()
            }
        }
    }
}

// async fn handle(&self) -> Result<(), HandleError> {
//     // With a socket ready, we can go ahead and read a packet. This will
//     // block until one is received.
//     let mut req_buffer = BytePacketBuffer::default();

//     // The `recv_from` function will write the data into the provided buffer,
//     // and return the length of the data read as well as the source address.
//     // We're not interested in the length, but we need to keep track of the
//     // source in order to send our reply later on.
//     let (_, src) = self.socket.recv_from(&mut req_buffer.buf).await?;
//     tracing::debug!("requested by {:?}", src.ip());

//     // Next, `DnsPacket::from_buffer` is used to parse the raw bytes into
//     // a `DnsPacket`.
//     let mut request = DnsPacket::try_from(req_buffer)?;

//     let mut tx = self.database.begin().await?;

//     // Create and initialize the response packet
//     let mut packet = DnsPacket::default();
//     packet.header.inner.id = request.header.inner.id;
//     packet.header.inner.recursion_desired = true;
//     packet.header.inner.recursion_available = true;
//     packet.header.inner.response = true;

//     // In the normal case, exactly one question is present
//     if let Some(question) = request.questions.pop() {
//         tracing::debug!("query: {question:?}");

//         if crate::model::blocklist::is_blocked(&mut tx, &src.ip(), &question.name).await? {
//             tracing::error!("qname {} is blocked for {src:?}", question.name);
//             packet.header.inner.response_code = ResponseCode::NameError;
//         } else if let Some(found) =
//             crate::model::record::find(&mut tx, question.qtype, &question.name).await?
//         {
//             tracing::debug!("{:?} {} found in cache", question.qtype, question.name);
//             packet.questions.push(question);
//             packet.header.inner.response_code = ResponseCode::NoError;

//             packet.answers.push(found);
//         } else {
//             tracing::debug!(
//                 "{:?} {} not found in cache, resolving",
//                 question.qtype,
//                 question.name
//             );
//             // Since all is set up and as expected, the query can be forwarded to the
//             // target server. There's always the possibility that the query will
//             // fail, in which case the `SERVFAIL` response code is set to indicate
//             // as much to the client. If rather everything goes as planned, the
//             // question and response records as copied into our response packet.
//             match self.lookup.execute(&question.name, question.qtype).await {
//                 Ok(result) => {
//                     packet.questions.push(question);
//                     packet.header.inner.response_code = result.header.inner.response_code;

//                     for rec in result.answers {
//                         tracing::debug!("answer: {rec:?}");

//                         match crate::model::record::persist(&mut tx, &rec).await {
//                             Ok(_) => tracing::debug!("persisted in cache"),
//                             Err(err) => tracing::error!("unable to persist in cache: {err:?}"),
//                         };

//                         packet.answers.push(rec);
//                     }
//                     for rec in result.authorities {
//                         tracing::debug!("authority: {rec:?}");
//                         packet.authorities.push(rec);
//                     }
//                     for rec in result.resources {
//                         tracing::debug!("resource: {rec:?}");
//                         packet.resources.push(rec);
//                     }
//                 }
//                 Err(error) => {
//                     tracing::error!("unable to lookup question: {error:?}");
//                     packet.header.inner.response_code = ResponseCode::ServerFailure;
//                 }
//             }
//         }
//     }
//     // Being mindful of how unreliable input data from arbitrary senders can be, we
//     // need make sure that a question is actually present. If not, we return `FORMERR`
//     // to indicate that the sender made something wrong.
//     else {
//         packet.header.inner.response_code = ResponseCode::FormatError;
//     }

//     // The only thing remaining is to encode our response and send it off!
//     let res_buffer = packet.create_buffer()?;

//     let len = res_buffer.pos();
//     let data = res_buffer.get_range(0, len)?;

//     self.socket.send_to(data, src).await?;

//     match tx.commit().await {
//         Ok(_) => {}
//         Err(err) => tracing::error!("unable to commit transaction: {err:?}"),
//     };

//     Ok(())
// }

#[cfg(test)]
mod tests {
    use super::DnsHandler;
    use crate::service::{blocklist::MockBlocklistService, lookup::MockLookupService};
    use donos_proto::{
        buffer::BytePacketBuffer,
        packet::{DnsPacket, QueryType},
    };
    use donos_server::{prelude::Message, Handler};
    use similar_asserts::assert_eq;
    use std::{
        net::{Ipv4Addr, SocketAddr, SocketAddrV4},
        sync::Arc,
    };

    fn socket_address() -> SocketAddr {
        SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 1, 0, 1), 42))
    }

    fn load_message(path: &str) -> Message {
        let address = socket_address();
        let (buffer, size) = load_buffer(path);

        Message {
            address,
            buffer,
            size,
        }
    }

    fn load_buffer(path: &str) -> ([u8; 512], usize) {
        let mut buffer = [0u8; 512];
        let content = std::fs::read(path).unwrap();
        content.iter().enumerate().for_each(|(idx, value)| {
            buffer[idx] = *value;
        });
        (buffer, content.len())
    }

    fn assert_same_messages((buffer, size): ([u8; 512], usize), result: Option<Message>) {
        if let Some(msg) = result {
            let received = BytePacketBuffer::new(msg.buffer);
            let received = DnsPacket::try_from(received).unwrap();
            let expected = BytePacketBuffer::new(buffer);
            let expected = DnsPacket::try_from(expected).unwrap();
            assert_eq!(expected, received);
            assert_eq!(size, msg.size);
        } else {
            panic!("should return a message");
        }
    }

    #[tokio::test]
    async fn should_resolve_query() {
        let input = load_message("assets/query_a_perducom_request.bin");
        let expected = load_buffer("assets/query_a_perducom_response.bin");
        let expected_packet = BytePacketBuffer::new(expected.0.clone());
        let mut expected_packet = DnsPacket::try_from(expected_packet).unwrap();
        expected_packet.header.inner.id = 1212;
        let blocklist = Arc::new(MockBlocklistService::default());
        let lookup = Arc::new(MockLookupService::default().with_query(
            "perdu.com",
            QueryType::A,
            expected_packet,
        ));
        let result = DnsHandler::new(blocklist, lookup).handle(input).await;
        if let Some(msg) = result {
            assert_eq!(msg.buffer, expected.0, "response should have same content");
            assert_eq!(msg.size, expected.1, "response should have same size");
        } else {
            panic!("should return a message");
        }
    }

    #[tokio::test]
    async fn should_block_query() {
        let input = load_message("assets/query_a_undefined_request.bin");
        let expected = load_buffer("assets/query_a_undefined_response.bin");
        let expected_buffer = BytePacketBuffer::new(expected.0.clone());
        let mut expected_packet = DnsPacket::try_from(expected_buffer).unwrap();
        expected_packet.header.inner.id = 1212;
        let blocklist = Arc::new(MockBlocklistService::default().with_domain("foo.bar.baz"));
        let lookup = Arc::new(MockLookupService::default());
        let result = DnsHandler::new(blocklist, lookup).handle(input).await;

        assert_same_messages(load_buffer("assets/query_a_undefined_response.bin"), result);
        // if let Some(msg) = result {
        //     let received = BytePacketBuffer::new(msg.buffer);
        //     let received = DnsPacket::try_from(received).unwrap();
        //     let expected = BytePacketBuffer::new(msg.buffer);
        //     let expected = DnsPacket::try_from(expected).unwrap();
        //     assert_eq!(received, expected);
        // } else {
        //     panic!("should return a message");
        // }
    }
}

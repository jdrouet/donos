pub mod buffer;
pub mod packet;

#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;

    fn copy_to(source: &[u8], target: &mut [u8]) {
        for (idx, val) in source.iter().enumerate() {
            target[idx] = *val;
        }
    }

    #[test]
    fn should_read_googlecom_query_packet() {
        let mut buffer = crate::buffer::BytePacketBuffer::default();
        copy_to(
            include_bytes!("../data/googlecom_query.bin"),
            &mut buffer.buf,
        );

        let packet = crate::packet::DnsPacket::try_from(buffer.clone()).unwrap();
        assert_eq!(packet.header.inner.id, 38005);
        assert!(packet.header.inner.recursion_desired);
        assert!(!packet.header.inner.truncated_message);

        assert_eq!(packet.questions.len(), 1);
        assert_eq!(packet.questions[0].name, "google.com");
        assert_eq!(packet.questions[0].qtype, crate::packet::QueryType::A);

        assert!(packet.answers.is_empty());
        assert!(packet.authorities.is_empty());
        assert!(packet.resources.is_empty());

        let mut packet = packet;
        let created = packet.create_buffer().unwrap();
        assert_eq!(buffer.buf, created.buf);
    }

    #[test]
    fn should_read_googlecom_response_packet() {
        let mut buffer = crate::buffer::BytePacketBuffer::default();
        copy_to(
            include_bytes!("../data/googlecom_response.bin"),
            &mut buffer.buf,
        );

        let packet = crate::packet::DnsPacket::try_from(buffer.clone()).unwrap();
        assert_eq!(packet.header.inner.id, 38005);
        assert!(packet.header.inner.recursion_desired);
        assert!(!packet.header.inner.truncated_message);

        assert_eq!(packet.questions.len(), 1);
        assert_eq!(packet.questions[0].name, "google.com");
        assert_eq!(packet.questions[0].qtype, crate::packet::QueryType::A);

        assert_eq!(packet.answers.len(), 1);
        assert_eq!(
            packet.answers[0],
            crate::packet::record::Record::A {
                domain: String::from("google.com"),
                addr: Ipv4Addr::new(172, 217, 20, 206),
                ttl: 8
            }
        );

        assert!(packet.authorities.is_empty());
        assert!(packet.resources.is_empty());

        let mut packet = packet;
        let created = packet.create_buffer().unwrap();
        assert_eq!(buffer.buf, created.buf);
    }

    #[test]
    fn should_read_appdatadoghqcom_query_packet() {
        let mut buffer = crate::buffer::BytePacketBuffer::default();
        copy_to(
            include_bytes!("../data/appdatadoghqcom_query.bin"),
            &mut buffer.buf,
        );

        let packet = crate::packet::DnsPacket::try_from(buffer.clone()).unwrap();
        assert_eq!(packet.header.inner.id, 45838);
        assert!(packet.header.inner.recursion_desired);
        assert!(!packet.header.inner.truncated_message);

        assert_eq!(packet.questions.len(), 1);
        assert_eq!(packet.questions[0].name, "app.datadoghq.com");
        assert_eq!(packet.questions[0].qtype, crate::packet::QueryType::A);

        assert!(packet.answers.is_empty());
        assert!(packet.authorities.is_empty());
        assert!(packet.resources.is_empty());

        let mut packet = packet;
        let created = packet.create_buffer().unwrap();
        assert_eq!(buffer.buf, created.buf);
    }

    #[test]
    fn should_read_appdatadoghqcom_response_packet() {
        let mut buffer = crate::buffer::BytePacketBuffer::default();
        copy_to(
            include_bytes!("../data/appdatadoghqcom_response.bin"),
            &mut buffer.buf,
        );

        let packet = crate::packet::DnsPacket::try_from(buffer.clone()).unwrap();
        assert_eq!(packet.header.inner.id, 45838);
        assert!(packet.header.inner.recursion_desired);
        assert!(!packet.header.inner.truncated_message);

        assert_eq!(packet.questions.len(), 1);
        assert_eq!(packet.questions[0].name, "app.datadoghq.com");
        assert_eq!(packet.questions[0].qtype, crate::packet::QueryType::A);

        assert_eq!(packet.answers.len(), 9);
        assert_eq!(
            packet.answers,
            vec![
                crate::packet::record::Record::CNAME {
                    domain: String::from("app.datadoghq.com"),
                    host: String::from(
                        "alb-web-2019-shard0-1497967001.us-east-1.elb.amazonaws.com"
                    ),
                    ttl: 39,
                },
                crate::packet::record::Record::A {
                    domain: String::from(
                        "alb-web-2019-shard0-1497967001.us-east-1.elb.amazonaws.com"
                    ),
                    addr: Ipv4Addr::new(3, 233, 151, 184),
                    ttl: 60,
                },
                crate::packet::record::Record::A {
                    domain: String::from(
                        "alb-web-2019-shard0-1497967001.us-east-1.elb.amazonaws.com"
                    ),
                    addr: Ipv4Addr::new(3, 233, 150, 239),
                    ttl: 60,
                },
                crate::packet::record::Record::A {
                    domain: String::from(
                        "alb-web-2019-shard0-1497967001.us-east-1.elb.amazonaws.com"
                    ),
                    addr: Ipv4Addr::new(3, 233, 151, 138),
                    ttl: 60,
                },
                crate::packet::record::Record::A {
                    domain: String::from(
                        "alb-web-2019-shard0-1497967001.us-east-1.elb.amazonaws.com"
                    ),
                    addr: Ipv4Addr::new(3, 233, 150, 225),
                    ttl: 60,
                },
                crate::packet::record::Record::A {
                    domain: String::from(
                        "alb-web-2019-shard0-1497967001.us-east-1.elb.amazonaws.com"
                    ),
                    addr: Ipv4Addr::new(3, 233, 151, 157),
                    ttl: 60,
                },
                crate::packet::record::Record::A {
                    domain: String::from(
                        "alb-web-2019-shard0-1497967001.us-east-1.elb.amazonaws.com"
                    ),
                    addr: Ipv4Addr::new(3, 233, 150, 30),
                    ttl: 60,
                },
                crate::packet::record::Record::A {
                    domain: String::from(
                        "alb-web-2019-shard0-1497967001.us-east-1.elb.amazonaws.com"
                    ),
                    addr: Ipv4Addr::new(3, 233, 150, 36),
                    ttl: 60,
                },
                crate::packet::record::Record::A {
                    domain: String::from(
                        "alb-web-2019-shard0-1497967001.us-east-1.elb.amazonaws.com"
                    ),
                    addr: Ipv4Addr::new(3, 233, 151, 128),
                    ttl: 60,
                }
            ]
        );

        assert!(packet.authorities.is_empty());
        assert!(packet.resources.is_empty());

        let mut packet = packet;
        let created = packet.create_buffer().unwrap();
        assert_eq!(buffer.buf, created.buf);
    }
}

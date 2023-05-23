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
    fn should_read_response_packet() {
        let mut buffer = crate::buffer::BytePacketBuffer::default();
        copy_to(
            include_bytes!("../data/response_packet.bin"),
            &mut buffer.buf,
        );

        let packet = crate::packet::DnsPacket::try_from(buffer).unwrap();
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
    }
}

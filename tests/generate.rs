use donos_parser::buffer::BytePacketBuffer;
use donos_parser::packet::question::Question;
use donos_parser::packet::{DnsPacket, QueryType};
use tokio::net::UdpSocket;

async fn exchange(buffer: &[u8], port: u16) -> Option<(usize, BytePacketBuffer)> {
    let socket = UdpSocket::bind(("0.0.0.0", port)).await.unwrap();
    socket.send_to(buffer, "1.1.1.1:53").await.unwrap();

    let mut response = BytePacketBuffer::default();

    let duration = std::time::Duration::from_secs(2);
    let process = socket.recv_from(&mut response.buf);
    match tokio::time::timeout(duration, process).await {
        Ok(Ok((size, _))) => Some((size, response)),
        Ok(_) => panic!("couldn't read"),
        Err(_) => None,
    }
}

async fn exchange_and_save(name: &str, packet: DnsPacket, port: u16) {
    println!("request: {packet:#?}");
    let buffer = packet.create_buffer().unwrap();

    let fname = format!("assets/{name}_request.bin");
    std::fs::write(&fname, &buffer.buf[0..buffer.pos]).unwrap();

    let (size, response) = exchange(&buffer.buf[0..buffer.pos], port).await.unwrap();

    let fname = format!("assets/{name}_response.bin");
    std::fs::write(fname, &response.buf[0..size]).unwrap();

    let packet = DnsPacket::try_from(response).unwrap();

    println!("response: {packet:#?}");
}

#[tokio::test]
#[ignore = "request without question doesn't answer"]
async fn without_question() {
    let mut packet = DnsPacket::default();

    packet.header.id = 1;
    packet.header.recursion_desired = true;

    let buffer = packet.create_buffer().unwrap();
    let response = exchange(&buffer.buf[0..buffer.pos], 43210).await;

    assert!(response.is_none());
}

#[tokio::test]
#[cfg_attr(not(feature = "generate"), ignore = "feature \"generate\" not enabled")]
async fn simple_a_query() {
    let mut packet = DnsPacket::default();

    packet.header.id = 2;
    packet.header.recursion_desired = true;
    packet
        .questions
        .push(Question::new("perdu.com".into(), QueryType::A));

    exchange_and_save("query_a_perducom", packet, 43211).await;
}

#[tokio::test]
#[cfg_attr(not(feature = "generate"), ignore = "feature \"generate\" not enabled")]
async fn simple_cname_query() {
    let mut packet = DnsPacket::default();

    packet.header.id = 3;
    packet.header.recursion_desired = true;
    packet
        .questions
        .push(Question::new("perdu.com".into(), QueryType::CNAME));

    exchange_and_save("query_cname_perducom", packet, 43212).await;
}

/// Out of this test, we can notice that the DNS server only answers the first question
#[tokio::test]
#[cfg_attr(not(feature = "generate"), ignore = "feature \"generate\" not enabled")]
async fn multiple_question_query() {
    let mut packet = DnsPacket::default();

    packet.header.id = 4;
    packet.header.recursion_desired = true;
    packet
        .questions
        .push(Question::new("perdu.com".into(), QueryType::A));
    packet
        .questions
        .push(Question::new("perdu.com".into(), QueryType::AAAA));
    packet
        .questions
        .push(Question::new("perdu.com".into(), QueryType::CNAME));

    exchange_and_save("query_multiple_question", packet, 43213).await;
}

/// Out of this test, we can notice that the DNS server only answers the first question
#[tokio::test]
#[cfg_attr(not(feature = "generate"), ignore = "feature \"generate\" not enabled")]
async fn multiple_answers_query() {
    let mut packet = DnsPacket::default();

    packet.header.id = 5;
    packet.header.recursion_desired = true;
    packet
        .questions
        .push(Question::new("app.datadoghq.com".into(), QueryType::A));

    exchange_and_save("query_multiple_answers", packet, 43214).await;
}

#[tokio::test]
#[cfg_attr(not(feature = "generate"), ignore = "feature \"generate\" not enabled")]
async fn undefined_a_query() {
    let mut packet = DnsPacket::default();

    packet.header.id = 6;
    packet.header.recursion_desired = true;
    packet
        .questions
        .push(Question::new("foo.bar.baz".into(), QueryType::A));

    exchange_and_save("query_a_undefined", packet, 43215).await;
}

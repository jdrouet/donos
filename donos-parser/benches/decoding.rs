use criterion::{criterion_group, criterion_main, Criterion};
use donos_parser::{buffer::BytePacketBuffer, packet::DnsPacket};

const QUERY_PACKET: &[u8] = include_bytes!("../data/googlecom_query.bin");
const RESPONSE_PACKET: &[u8] = include_bytes!("../data/googlecom_response.bin");

fn copy_to(source: &[u8], target: &mut [u8]) {
    for (idx, val) in source.iter().enumerate() {
        target[idx] = *val;
    }
}

fn decoding(packet: &[u8]) {
    let mut buffer = BytePacketBuffer::default();
    copy_to(packet, &mut buffer.buf);
    let _ = DnsPacket::try_from(buffer).unwrap();
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("decoding query packet", |b| {
        b.iter(|| decoding(QUERY_PACKET))
    });
    c.bench_function("decoding response packet", |b| {
        b.iter(|| decoding(RESPONSE_PACKET))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

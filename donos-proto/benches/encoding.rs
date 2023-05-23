use criterion::{black_box, criterion_group, criterion_main, Criterion};
use donos_proto::{BytePacketBuffer, DnsPacket};

const QUERY_PACKET: &[u8] = include_bytes!("../data/query_packet.bin");
const RESPONSE_PACKET: &[u8] = include_bytes!("../data/response_packet.bin");

fn copy_to(source: &[u8], target: &mut [u8]) {
    for (idx, val) in source.iter().enumerate() {
        target[idx] = *val;
    }
}

fn prepare(packet: &[u8]) -> DnsPacket {
    let mut buffer = BytePacketBuffer::default();
    copy_to(packet, &mut buffer.buf);
    DnsPacket::try_from(buffer).unwrap()
}

fn encoding(mut packet: DnsPacket) {
    let _buffer = packet.create_buffer().unwrap();
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("encoding query packet", |b| {
        let packet = prepare(QUERY_PACKET);
        b.iter(|| encoding(black_box(packet.clone())))
    });
    c.bench_function("encoding response packet", |b| {
        let packet = prepare(RESPONSE_PACKET);
        b.iter(|| encoding(black_box(packet.clone())))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

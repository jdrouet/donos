#![no_main]

use donos_proto::{BytePacketBuffer, DnsPacket};

libfuzzer_sys::fuzz_target!(|buffer: BytePacketBuffer| {
    let mut buffer = buffer;
    let _ = DnsPacket::from_buffer(&mut buffer);
});

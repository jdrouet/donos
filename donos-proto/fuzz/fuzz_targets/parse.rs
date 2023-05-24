#![no_main]

use donos_proto::{buffer::BytePacketBuffer, packet::DnsPacket};
use std::convert::TryFrom;

libfuzzer_sys::fuzz_target!(|buffer: BytePacketBuffer| {
    let _ = DnsPacket::try_from(buffer);
});

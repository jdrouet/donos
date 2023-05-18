use crate::buffer::*;
use crate::QueryType;
use std::net::{Ipv4Addr, Ipv6Addr};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DnsRecordA {
    pub domain: String,
    pub addr: Ipv4Addr,
    pub ttl: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DnsRecordAAAA {
    domain: String,
    addr: Ipv6Addr,
    ttl: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DnsRecordNS {
    domain: String,
    host: String,
    ttl: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DnsRecordCNAME {
    domain: String,
    host: String,
    ttl: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DnsRecordMX {
    domain: String,
    priority: u16,
    host: String,
    ttl: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DnsRecordUnknown {
    domain: String,
    qtype: u16,
    data_len: u16,
    ttl: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[allow(clippy::upper_case_acronyms)]
pub enum DnsRecord {
    Unknown(DnsRecordUnknown), // 0
    A(DnsRecordA),             // 1
    NS(DnsRecordNS),           // 2
    CNAME(DnsRecordCNAME),     // 5
    MX(DnsRecordMX),           // 15
    AAAA(DnsRecordAAAA),       // 28
}

impl DnsRecord {
    pub fn read(buffer: &mut BytePacketBuffer) -> Result<DnsRecord, ReaderError> {
        // NAME a domain name to which this resource record pertains.
        let mut domain = String::new();
        buffer.read_qname(&mut domain)?;

        // TYPE two octets containing one of the RR type codes.
        // This field specifies the meaning of the data in the RDATA field.
        let qtype_num = buffer.read_u16()?;
        let qtype = QueryType::from_num(qtype_num);

        // CLASS two octets which specify the class of the data in the RDATA field.
        let _qclass = buffer.read_u16()?;

        // TTL a 32 bit unsigned integer that specifies the time interval (in seconds)
        // that the resource record may be cached before it should be discarded.
        // Zero values are interpreted to mean that the RR can only be used for
        // the transaction in progress, and should not be cached.
        let ttl = buffer.read_u32()?;

        // RDLENGTH an unsigned 16 bit integer that specifies the length in octets of the RDATA field.
        let data_len = buffer.read_u16()?;

        match qtype {
            QueryType::A => {
                let raw_addr = buffer.read_u32()?;
                let addr = Ipv4Addr::new(
                    ((raw_addr >> 24) & 0xFF) as u8,
                    ((raw_addr >> 16) & 0xFF) as u8,
                    ((raw_addr >> 8) & 0xFF) as u8,
                    (raw_addr & 0xFF) as u8,
                );

                Ok(DnsRecord::A(DnsRecordA { domain, addr, ttl }))
            }
            QueryType::AAAA => {
                let raw_addr1 = buffer.read_u32()?;
                let raw_addr2 = buffer.read_u32()?;
                let raw_addr3 = buffer.read_u32()?;
                let raw_addr4 = buffer.read_u32()?;
                let addr = Ipv6Addr::new(
                    ((raw_addr1 >> 16) & 0xFFFF) as u16,
                    (raw_addr1 & 0xFFFF) as u16,
                    ((raw_addr2 >> 16) & 0xFFFF) as u16,
                    (raw_addr2 & 0xFFFF) as u16,
                    ((raw_addr3 >> 16) & 0xFFFF) as u16,
                    (raw_addr3 & 0xFFFF) as u16,
                    ((raw_addr4 >> 16) & 0xFFFF) as u16,
                    (raw_addr4 & 0xFFFF) as u16,
                );

                Ok(DnsRecord::AAAA(DnsRecordAAAA { domain, addr, ttl }))
            }
            QueryType::NS => {
                let mut ns = String::new();
                buffer.read_qname(&mut ns)?;

                Ok(DnsRecord::NS(DnsRecordNS {
                    domain,
                    host: ns,
                    ttl,
                }))
            }
            QueryType::CNAME => {
                let mut cname = String::new();
                buffer.read_qname(&mut cname)?;

                Ok(DnsRecord::CNAME(DnsRecordCNAME {
                    domain,
                    host: cname,
                    ttl,
                }))
            }
            QueryType::MX => {
                let priority = buffer.read_u16()?;
                let mut mx = String::new();
                buffer.read_qname(&mut mx)?;

                Ok(DnsRecord::MX(DnsRecordMX {
                    domain,
                    priority,
                    host: mx,
                    ttl,
                }))
            }
            QueryType::Unknown(_) => {
                buffer.step(data_len as usize)?;

                Ok(DnsRecord::Unknown(DnsRecordUnknown {
                    domain,
                    qtype: qtype_num,
                    data_len,
                    ttl,
                }))
            }
        }
    }

    pub fn write(&self, buffer: &mut BytePacketBuffer) -> Result<usize, WriterError> {
        let start_pos = buffer.pos();

        match *self {
            DnsRecord::A(DnsRecordA {
                ref domain,
                ref addr,
                ttl,
            }) => {
                buffer.write_qname(domain)?;
                buffer.write_u16(QueryType::A.to_num())?;
                buffer.write_u16(1)?;
                buffer.write_u32(ttl)?;
                buffer.write_u16(4)?;

                let octets = addr.octets();
                buffer.write_u8(octets[0])?;
                buffer.write_u8(octets[1])?;
                buffer.write_u8(octets[2])?;
                buffer.write_u8(octets[3])?;
            }
            DnsRecord::NS(DnsRecordNS {
                ref domain,
                ref host,
                ttl,
            }) => {
                buffer.write_qname(domain)?;
                buffer.write_u16(QueryType::NS.to_num())?;
                buffer.write_u16(1)?;
                buffer.write_u32(ttl)?;

                let pos = buffer.pos();
                buffer.write_u16(0)?;

                buffer.write_qname(host)?;

                let size = buffer.pos() - (pos + 2);
                buffer.set_u16(pos, size as u16)?;
            }
            DnsRecord::CNAME(DnsRecordCNAME {
                ref domain,
                ref host,
                ttl,
            }) => {
                buffer.write_qname(domain)?;
                buffer.write_u16(QueryType::CNAME.to_num())?;
                buffer.write_u16(1)?;
                buffer.write_u32(ttl)?;

                let pos = buffer.pos();
                buffer.write_u16(0)?;

                buffer.write_qname(host)?;

                let size = buffer.pos() - (pos + 2);
                buffer.set_u16(pos, size as u16)?;
            }
            DnsRecord::MX(DnsRecordMX {
                ref domain,
                priority,
                ref host,
                ttl,
            }) => {
                buffer.write_qname(domain)?;
                buffer.write_u16(QueryType::MX.to_num())?;
                buffer.write_u16(1)?;
                buffer.write_u32(ttl)?;

                let pos = buffer.pos();
                buffer.write_u16(0)?;

                buffer.write_u16(priority)?;
                buffer.write_qname(host)?;

                let size = buffer.pos() - (pos + 2);
                buffer.set_u16(pos, size as u16)?;
            }
            DnsRecord::AAAA(DnsRecordAAAA {
                ref domain,
                ref addr,
                ttl,
            }) => {
                buffer.write_qname(domain)?;
                buffer.write_u16(QueryType::AAAA.to_num())?;
                buffer.write_u16(1)?;
                buffer.write_u32(ttl)?;
                buffer.write_u16(16)?;

                for octet in &addr.segments() {
                    buffer.write_u16(*octet)?;
                }
            }
            DnsRecord::Unknown { .. } => {
                println!("Skipping record: {:?}", self);
            }
        }

        Ok(buffer.pos() - start_pos)
    }
}
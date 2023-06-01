use super::QueryType;
use crate::buffer::reader::ReaderError;
use crate::buffer::writer::WriterError;
use crate::buffer::BytePacketBuffer;
use std::net::{Ipv4Addr, Ipv6Addr};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[allow(clippy::upper_case_acronyms)]
pub enum Record {
    Unknown {
        domain: String,
        qtype: u16,
        data_len: u16,
        ttl: u32,
    }, // 0
    A {
        domain: String,
        addr: Ipv4Addr,
        ttl: u32,
    }, // 1
    NS {
        domain: String,
        host: String,
        ttl: u32,
    }, // 2
    CNAME {
        domain: String,
        host: String,
        ttl: u32,
    }, // 5
    MX {
        domain: String,
        priority: u16,
        host: String,
        ttl: u32,
    }, // 15
    AAAA {
        domain: String,
        addr: Ipv6Addr,
        ttl: u32,
    }, // 28
}

impl Record {
    pub fn ttl(&self) -> u32 {
        match self {
            Self::A { ttl, .. } => *ttl,
            Self::AAAA { ttl, .. } => *ttl,
            Self::CNAME { ttl, .. } => *ttl,
            Self::MX { ttl, .. } => *ttl,
            Self::NS { ttl, .. } => *ttl,
            Self::Unknown { ttl, .. } => *ttl,
        }
    }

    pub fn delayed_ttl(&self, ttl: u32) -> Self {
        match self {
            Self::A { domain, addr, .. } => Self::A {
                domain: domain.clone(),
                addr: addr.clone(),
                ttl,
            },
            Self::AAAA { domain, addr, .. } => Self::AAAA {
                domain: domain.clone(),
                addr: addr.clone(),
                ttl,
            },
            Self::CNAME { domain, host, .. } => Self::CNAME {
                domain: domain.clone(),
                host: host.clone(),
                ttl,
            },
            Self::MX {
                domain,
                priority,
                host,
                ..
            } => Self::MX {
                domain: domain.clone(),
                priority: *priority,
                host: host.clone(),
                ttl,
            },
            Self::NS { domain, host, .. } => Self::NS {
                domain: domain.clone(),
                host: host.clone(),
                ttl,
            },
            Self::Unknown {
                domain,
                qtype,
                data_len,
                ..
            } => Self::Unknown {
                domain: domain.clone(),
                qtype: *qtype,
                data_len: *data_len,
                ttl,
            },
        }
    }

    pub fn read(buffer: &mut BytePacketBuffer) -> Result<Record, ReaderError> {
        // NAME a domain name to which this resource record pertains.
        let domain = buffer.read_qname()?;

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

                Ok(Record::A { domain, addr, ttl })
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

                Ok(Record::AAAA { domain, addr, ttl })
            }
            QueryType::NS => {
                let host = buffer.read_qname()?;

                Ok(Record::NS { domain, host, ttl })
            }
            QueryType::CNAME => {
                let host = buffer.read_qname()?;

                Ok(Record::CNAME { domain, host, ttl })
            }
            QueryType::MX => {
                let priority = buffer.read_u16()?;
                let host = buffer.read_qname()?;

                Ok(Record::MX {
                    domain,
                    priority,
                    host,
                    ttl,
                })
            }
            QueryType::Unknown(_) => {
                buffer.step(data_len as usize)?;

                Ok(Record::Unknown {
                    domain,
                    qtype: qtype_num,
                    data_len,
                    ttl,
                })
            }
        }
    }

    pub fn write(&self, buffer: &mut BytePacketBuffer) -> Result<usize, WriterError> {
        let start_pos = buffer.pos();

        match *self {
            Record::A {
                ref domain,
                ref addr,
                ttl,
            } => {
                buffer.write_qname(domain)?;
                buffer.write_u16(QueryType::A.into_num())?;
                buffer.write_u16(1)?;
                buffer.write_u32(ttl)?;
                buffer.write_u16(4)?;

                let octets = addr.octets();
                buffer.write_u8(octets[0])?;
                buffer.write_u8(octets[1])?;
                buffer.write_u8(octets[2])?;
                buffer.write_u8(octets[3])?;
            }
            Record::NS {
                ref domain,
                ref host,
                ttl,
            } => {
                buffer.write_qname(domain)?;
                buffer.write_u16(QueryType::NS.into_num())?;
                buffer.write_u16(1)?;
                buffer.write_u32(ttl)?;

                let pos = buffer.pos();
                buffer.write_u16(0)?;

                buffer.write_qname(host)?;

                let size = buffer.pos() - (pos + 2);
                buffer.set_u16(pos, size as u16)?;
            }
            Record::CNAME {
                ref domain,
                ref host,
                ttl,
            } => {
                buffer.write_qname(domain)?;
                buffer.write_u16(QueryType::CNAME.into_num())?;
                buffer.write_u16(1)?;
                buffer.write_u32(ttl)?;

                let pos = buffer.pos();
                buffer.write_u16(0)?;

                buffer.write_qname(host)?;

                let size = buffer.pos() - (pos + 2);
                buffer.set_u16(pos, size as u16)?;
            }
            Record::MX {
                ref domain,
                priority,
                ref host,
                ttl,
            } => {
                buffer.write_qname(domain)?;
                buffer.write_u16(QueryType::MX.into_num())?;
                buffer.write_u16(1)?;
                buffer.write_u32(ttl)?;

                let pos = buffer.pos();
                buffer.write_u16(0)?;

                buffer.write_u16(priority)?;
                buffer.write_qname(host)?;

                let size = buffer.pos() - (pos + 2);
                buffer.set_u16(pos, size as u16)?;
            }
            Record::AAAA {
                ref domain,
                ref addr,
                ttl,
            } => {
                buffer.write_qname(domain)?;
                buffer.write_u16(QueryType::AAAA.into_num())?;
                buffer.write_u16(1)?;
                buffer.write_u32(ttl)?;
                buffer.write_u16(16)?;

                for octet in &addr.segments() {
                    buffer.write_u16(*octet)?;
                }
            }
            Record::Unknown { .. } => {
                println!("Skipping record: {:?}", self);
            }
        }

        Ok(buffer.pos() - start_pos)
    }
}

use std::net::{Ipv4Addr, Ipv6Addr};

#[derive(Debug)]
pub struct InvalidResponseCode(pub u8);

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ResponseCode {
    /// No error condition
    NoError = 0,
    /// Format error - The name server was unable to interpret the query.
    FormatError = 1,
    /// Server failure - The name server was unable to process this query due to a problem with the name server.
    ServerFailure = 2,
    /// Name Error - Meaningful only for responses from an authoritative name server,
    /// this code signifies that the domain name referenced in the query does not exist.
    /// Previously name NXDOMAIN
    NameError = 3,
    /// Not Implemented - The name server does not support the requested kind of query.
    NotImplemented = 4,
    /// Refused - The name server refuses to perform the specified operation for policy reasons.
    /// For example, a name server may not wish to provide the information to the particular requester,
    /// or a name server may not wish to perform a particular operation (e.g., zone transfer) for particular data.
    Refused = 5,
}

/// TODO Handle invalid values
impl ResponseCode {
    pub fn from_num(num: u8) -> ResponseCode {
        match num {
            1 => ResponseCode::FormatError,
            2 => ResponseCode::ServerFailure,
            3 => ResponseCode::NameError,
            4 => ResponseCode::NotImplemented,
            5 => ResponseCode::Refused,
            _ => ResponseCode::NoError,
        }
    }
}
#[derive(Clone, Debug)]
pub struct DnsHeader {
    /// A 16 bit identifier assigned by the program that
    /// generates any kind of query.  This identifier is copied
    /// the corresponding reply and can be used by the requester
    /// to match up replies to outstanding queries.
    pub id: u16, // 16 bits

    /// RD Recursion Desired - this bit may be set in a query and is copied into the response.
    /// If RD is set, it directs the name server to pursue the query recursively.
    /// Recursive query support is optional.
    pub recursion_desired: bool, // 1 bit
    /// TC TrunCation - specifies that this message was truncated due to length greater
    /// than that permitted on the transmission channel.
    pub truncated_message: bool, // 1 bit
    /// AA Authoritative Answer - this bit is valid in responses,
    /// and specifies that the responding name server is an authority
    /// for the domain name in question section.
    ///
    /// Note that the contents of the answer section may have multiple owner names because of aliases.
    /// The AA bit corresponds to the name which matches the query name,
    /// or the first owner name in the answer section.
    pub authoritative_answer: bool, // 1 bit
    /// OPCODE A four bit field that specifies kind of query in this message.
    /// This value is set by the originator of a query and copied into the response.
    /// The values are:
    ///   0               a standard query (QUERY)
    ///   1               an inverse query (IQUERY)
    ///   2               a server status request (STATUS)
    ///   3-15            reserved for future use
    pub opcode: u8, // 4 bits
    /// QR A one bit field that specifies whether this message is a query (0), or a response (1).
    pub response: bool, // 1 bit

    /// Response code - this 4 bit field is set as part of responses.
    pub response_code: ResponseCode, // 4 bits
    pub checking_disabled: bool, // 1 bit
    pub authed_data: bool,       // 1 bit
    /// Z Reserved for future use.  Must be zero in all queries and responses.
    pub z: bool, // 1 bit
    /// RA Recursion Available - this be is set or cleared in a response,
    /// and denotes whether recursive query support is available in the name server.
    pub recursion_available: bool, // 1 bit

    /// QDCOUNT an unsigned 16 bit integer specifying the number of entries in the question section.
    pub questions: u16, // 16 bits
    /// ANCOUNT an unsigned 16 bit integer specifying the number of resource records in the answer section.
    pub answers: u16, // 16 bits
    /// NSCOUNT an unsigned 16 bit integer specifying the number of name server resource records in the authority records section.
    pub authoritative_entries: u16, // 16 bits
    /// ARCOUNT an unsigned 16 bit integer specifying the number of resource records in the additional records section.
    pub resource_entries: u16, // 16 bits
}

impl DnsHeader {
    pub fn new() -> DnsHeader {
        DnsHeader {
            id: 0,

            recursion_desired: false,
            truncated_message: false,
            authoritative_answer: false,
            opcode: 0,
            response: false,

            response_code: ResponseCode::NoError,
            checking_disabled: false,
            authed_data: false,
            z: false,
            recursion_available: false,

            questions: 0,
            answers: 0,
            authoritative_entries: 0,
            resource_entries: 0,
        }
    }

    pub fn read(
        &mut self,
        buffer: &mut crate::buffer::BytePacketBuffer,
    ) -> Result<(), crate::buffer::ReaderError> {
        self.id = buffer.read_u16()?;

        let flags = buffer.read_u16()?;
        let head = (flags >> 8) as u8;
        self.recursion_desired = (head & (1 << 0)) > 0;
        self.truncated_message = (head & (1 << 1)) > 0;
        self.authoritative_answer = (head & (1 << 2)) > 0;
        self.opcode = (head >> 3) & 0x0F;
        self.response = (head & (1 << 7)) > 0;

        let tail = (flags & 0xFF) as u8;
        self.response_code = ResponseCode::from_num(tail & 0x0F);
        self.checking_disabled = (tail & (1 << 4)) > 0;
        self.authed_data = (tail & (1 << 5)) > 0;
        self.z = (tail & (1 << 6)) > 0;
        self.recursion_available = (tail & (1 << 7)) > 0;

        self.questions = buffer.read_u16()?;
        self.answers = buffer.read_u16()?;
        self.authoritative_entries = buffer.read_u16()?;
        self.resource_entries = buffer.read_u16()?;

        Ok(())
    }

    pub fn write(
        &self,
        buffer: &mut crate::buffer::BytePacketBuffer,
    ) -> Result<(), crate::buffer::WriterError> {
        buffer.write_u16(self.id)?;

        buffer.write_u8(
            (self.recursion_desired as u8)
                | ((self.truncated_message as u8) << 1)
                | ((self.authoritative_answer as u8) << 2)
                | (self.opcode << 3)
                | ((self.response as u8) << 7),
        )?;

        buffer.write_u8(
            (self.response_code as u8)
                | ((self.checking_disabled as u8) << 4)
                | ((self.authed_data as u8) << 5)
                | ((self.z as u8) << 6)
                | ((self.recursion_available as u8) << 7),
        )?;

        buffer.write_u16(self.questions)?;
        buffer.write_u16(self.answers)?;
        buffer.write_u16(self.authoritative_entries)?;
        buffer.write_u16(self.resource_entries)?;

        Ok(())
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Hash, Copy)]
#[allow(clippy::upper_case_acronyms)]
pub enum QueryType {
    Unknown(u16),
    /// a host address
    A, // 1
    /// an authoritative name server
    NS, // 2
    /// the canonical name for an alias
    CNAME, // 5
    /// mail exchange
    MX, // 15
    AAAA, // 28
}

impl QueryType {
    pub fn to_num(self) -> u16 {
        match self {
            QueryType::Unknown(x) => x,
            QueryType::A => 1,
            QueryType::NS => 2,
            QueryType::CNAME => 5,
            QueryType::MX => 15,
            QueryType::AAAA => 28,
        }
    }

    /// TODO Handle invalid values
    pub fn from_num(num: u16) -> QueryType {
        match num {
            1 => QueryType::A,
            2 => QueryType::NS,
            5 => QueryType::CNAME,
            15 => QueryType::MX,
            28 => QueryType::AAAA,
            _ => QueryType::Unknown(num),
        }
    }
}

/// CLASS fields appear in resource records.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum DnsClass {
    /// IN - the Internet
    Internet = 1,
    /// CS - the CSNET class (Obsolete - used only for examples in some obsolete RFCs)
    Csnet = 2,
    /// CH - the CHAOS class
    Chaos = 3,
    /// HS - Hesiod [Dyer 87]
    Hesiod = 4,
}

impl Default for DnsClass {
    fn default() -> Self {
        Self::Internet
    }
}

/// TODO Handle invalid values
impl DnsClass {
    fn from_num(value: u16) -> Self {
        match value {
            1 => Self::Internet,
            2 => Self::Csnet,
            3 => Self::Chaos,
            4 => Self::Hesiod,
            _other => Self::Internet,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DnsQuestion {
    /// QNAME a domain name represented as a sequence of labels,
    /// where each label consists of a length octet followed by that number of octets.
    /// The domain name terminates with the zero length octet for the null label of the root.
    /// Note that this field may be an odd number of octets; no padding is used.
    pub name: String,
    /// QTYPE a two octet code which specifies the type of the query.
    /// The values for this field include all codes valid for a TYPE field,
    /// together with some more general codes which can match more than one type of RR.
    pub qtype: QueryType,
    /// QCLASS a two octet code that specifies the class of the query.
    /// For example, the QCLASS field is IN for the Internet.
    pub qclass: DnsClass,
}

impl Default for DnsQuestion {
    fn default() -> Self {
        Self {
            name: String::default(),
            qtype: QueryType::Unknown(0),
            qclass: DnsClass::Internet,
        }
    }
}

impl DnsQuestion {
    pub fn new(name: String, qtype: QueryType) -> Self {
        Self {
            name,
            qtype,
            qclass: Default::default(),
        }
    }

    pub fn read(
        buffer: &mut crate::buffer::BytePacketBuffer,
    ) -> Result<Self, crate::buffer::ReaderError> {
        let mut name = String::new();
        buffer.read_qname(&mut name)?;
        let qtype = QueryType::from_num(buffer.read_u16()?); // qtype
        let qclass = DnsClass::from_num(buffer.read_u16()?); // class

        Ok(DnsQuestion {
            name,
            qtype,
            qclass,
        })
    }

    pub fn write(
        &self,
        buffer: &mut crate::buffer::BytePacketBuffer,
    ) -> Result<(), crate::buffer::WriterError> {
        buffer.write_qname(&self.name)?;

        let typenum = self.qtype.to_num();
        buffer.write_u16(typenum)?;
        buffer.write_u16(self.qclass as u16)?;

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[allow(clippy::upper_case_acronyms)]
pub enum DnsRecord {
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

impl DnsRecord {
    pub fn read(
        buffer: &mut crate::buffer::BytePacketBuffer,
    ) -> Result<DnsRecord, crate::buffer::ReaderError> {
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

                Ok(DnsRecord::A { domain, addr, ttl })
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

                Ok(DnsRecord::AAAA { domain, addr, ttl })
            }
            QueryType::NS => {
                let mut ns = String::new();
                buffer.read_qname(&mut ns)?;

                Ok(DnsRecord::NS {
                    domain,
                    host: ns,
                    ttl,
                })
            }
            QueryType::CNAME => {
                let mut cname = String::new();
                buffer.read_qname(&mut cname)?;

                Ok(DnsRecord::CNAME {
                    domain,
                    host: cname,
                    ttl,
                })
            }
            QueryType::MX => {
                let priority = buffer.read_u16()?;
                let mut mx = String::new();
                buffer.read_qname(&mut mx)?;

                Ok(DnsRecord::MX {
                    domain,
                    priority,
                    host: mx,
                    ttl,
                })
            }
            QueryType::Unknown(_) => {
                buffer.step(data_len as usize)?;

                Ok(DnsRecord::Unknown {
                    domain,
                    qtype: qtype_num,
                    data_len,
                    ttl,
                })
            }
        }
    }

    pub fn write(
        &self,
        buffer: &mut crate::buffer::BytePacketBuffer,
    ) -> Result<usize, crate::buffer::WriterError> {
        let start_pos = buffer.pos();

        match *self {
            DnsRecord::A {
                ref domain,
                ref addr,
                ttl,
            } => {
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
            DnsRecord::NS {
                ref domain,
                ref host,
                ttl,
            } => {
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
            DnsRecord::CNAME {
                ref domain,
                ref host,
                ttl,
            } => {
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
            DnsRecord::MX {
                ref domain,
                priority,
                ref host,
                ttl,
            } => {
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
            DnsRecord::AAAA {
                ref domain,
                ref addr,
                ttl,
            } => {
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

#[derive(Clone, Debug)]
pub struct DnsPacket {
    pub header: DnsHeader,
    pub questions: Vec<DnsQuestion>,
    pub answers: Vec<DnsRecord>,
    pub authorities: Vec<DnsRecord>,
    pub resources: Vec<DnsRecord>,
}

impl DnsPacket {
    pub fn new() -> DnsPacket {
        DnsPacket {
            header: DnsHeader::new(),
            questions: Vec::new(),
            answers: Vec::new(),
            authorities: Vec::new(),
            resources: Vec::new(),
        }
    }

    pub fn from_buffer(
        buffer: &mut crate::buffer::BytePacketBuffer,
    ) -> Result<DnsPacket, crate::buffer::ReaderError> {
        let mut result = DnsPacket::new();
        result.header.read(buffer)?;

        for _ in 0..result.header.questions {
            result.questions.push(DnsQuestion::read(buffer)?);
        }

        for _ in 0..result.header.answers {
            result.answers.push(DnsRecord::read(buffer)?);
        }
        for _ in 0..result.header.authoritative_entries {
            result.authorities.push(DnsRecord::read(buffer)?);
        }
        for _ in 0..result.header.resource_entries {
            result.resources.push(DnsRecord::read(buffer)?);
        }

        Ok(result)
    }

    pub fn write(
        &mut self,
        buffer: &mut crate::buffer::BytePacketBuffer,
    ) -> Result<(), crate::buffer::WriterError> {
        self.header.questions = self.questions.len() as u16;
        self.header.answers = self.answers.len() as u16;
        self.header.authoritative_entries = self.authorities.len() as u16;
        self.header.resource_entries = self.resources.len() as u16;

        self.header.write(buffer)?;

        for question in &self.questions {
            question.write(buffer)?;
        }
        for rec in &self.answers {
            rec.write(buffer)?;
        }
        for rec in &self.authorities {
            rec.write(buffer)?;
        }
        for rec in &self.resources {
            rec.write(buffer)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::buffer::BytePacketBuffer;
    use std::net::Ipv4Addr;

    fn copy_to(source: &[u8], target: &mut [u8]) {
        for (idx, val) in source.iter().enumerate() {
            target[idx] = *val;
        }
    }

    #[test]
    fn should_read_response_packet() {
        let mut buffer = BytePacketBuffer::new();
        copy_to(
            include_bytes!("../data/response_packet.bin"),
            &mut buffer.buf,
        );

        let packet = super::DnsPacket::from_buffer(&mut buffer).unwrap();
        assert_eq!(packet.header.id, 38005);
        assert_eq!(packet.header.recursion_desired, true);
        assert_eq!(packet.header.truncated_message, false);

        assert_eq!(packet.questions.len(), 1);
        assert_eq!(packet.questions[0].name, "google.com");
        assert_eq!(packet.questions[0].qtype, super::QueryType::A);

        assert_eq!(packet.answers.len(), 1);
        assert_eq!(
            packet.answers[0],
            crate::packet::DnsRecord::A {
                domain: String::from("google.com"),
                addr: Ipv4Addr::new(172, 217, 20, 206),
                ttl: 8
            }
        );

        assert!(packet.authorities.is_empty());
        assert!(packet.resources.is_empty());
    }
}

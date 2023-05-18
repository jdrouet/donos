use crate::buffer::{BytePacketBuffer, ReaderError, WriterError};

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

    pub fn read(buffer: &mut BytePacketBuffer) -> Result<Self, ReaderError> {
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

    pub fn write(&self, buffer: &mut BytePacketBuffer) -> Result<(), WriterError> {
        buffer.write_qname(&self.name)?;

        let typenum = self.qtype.to_num();
        buffer.write_u16(typenum)?;
        buffer.write_u16(self.qclass as u16)?;

        Ok(())
    }
}

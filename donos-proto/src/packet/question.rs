use super::QueryType;
use crate::buffer::reader::ReaderError;
use crate::buffer::writer::WriterError;
use crate::buffer::BytePacketBuffer;

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

impl TryFrom<u16> for DnsClass {
    type Error = ReaderError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Internet),
            2 => Ok(Self::Csnet),
            3 => Ok(Self::Chaos),
            4 => Ok(Self::Hesiod),
            other => Err(ReaderError::InvalidClass(other)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Question {
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

impl Default for Question {
    fn default() -> Self {
        Self {
            name: String::default(),
            qtype: QueryType::Unknown(0),
            qclass: DnsClass::Internet,
        }
    }
}

impl Question {
    pub fn new(name: String, qtype: QueryType) -> Self {
        Self {
            name,
            qtype,
            qclass: Default::default(),
        }
    }

    pub fn read(buffer: &mut BytePacketBuffer) -> Result<Self, ReaderError> {
        let name = buffer.read_qname()?;
        let qtype = QueryType::from_num(buffer.read_u16()?); // qtype
        let qclass = DnsClass::try_from(buffer.read_u16()?)?; // class

        Ok(Self {
            name,
            qtype,
            qclass,
        })
    }

    pub fn write(&self, buffer: &mut BytePacketBuffer) -> Result<(), WriterError> {
        buffer.write_qname(&self.name)?;

        let typenum = self.qtype.into_num();
        buffer.write_u16(typenum)?;
        buffer.write_u16(self.qclass as u16)?;

        Ok(())
    }
}

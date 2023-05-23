pub mod header;
pub mod question;
pub mod record;

use crate::buffer::{BytePacketBuffer, ReaderError, WriterError};

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
    pub fn into_num(self) -> u16 {
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

#[derive(Clone, Debug, Default)]
pub struct DnsPacket {
    pub header: header::Header,
    pub questions: Vec<question::Question>,
    pub answers: Vec<record::Record>,
    pub authorities: Vec<record::Record>,
    pub resources: Vec<record::Record>,
}

impl TryFrom<BytePacketBuffer> for DnsPacket {
    type Error = ReaderError;

    fn try_from(mut buffer: BytePacketBuffer) -> Result<Self, Self::Error> {
        let header = header::Header::read(&mut buffer)?;

        let mut questions = Vec::with_capacity(header.questions as usize);
        for _ in 0..header.questions {
            questions.push(question::Question::read(&mut buffer)?);
        }

        let mut answers = Vec::with_capacity(header.answers as usize);
        for _ in 0..header.answers {
            answers.push(record::Record::read(&mut buffer)?);
        }

        let mut authorities = Vec::with_capacity(header.authoritative_entries as usize);
        for _ in 0..header.authoritative_entries {
            authorities.push(record::Record::read(&mut buffer)?);
        }

        let mut resources = Vec::with_capacity(header.resource_entries as usize);
        for _ in 0..header.resource_entries {
            resources.push(record::Record::read(&mut buffer)?);
        }

        Ok(DnsPacket {
            header,
            questions,
            answers,
            authorities,
            resources,
        })
    }
}

impl DnsPacket {
    pub fn create_buffer(&mut self) -> Result<BytePacketBuffer, WriterError> {
        let mut buffer = BytePacketBuffer::default();
        self.header.questions = self.questions.len() as u16;
        self.header.answers = self.answers.len() as u16;
        self.header.authoritative_entries = self.authorities.len() as u16;
        self.header.resource_entries = self.resources.len() as u16;

        self.header.write(&mut buffer)?;

        for question in &self.questions {
            question.write(&mut buffer)?;
        }
        for rec in &self.answers {
            rec.write(&mut buffer)?;
        }
        for rec in &self.authorities {
            rec.write(&mut buffer)?;
        }
        for rec in &self.resources {
            rec.write(&mut buffer)?;
        }

        Ok(buffer)
    }
}

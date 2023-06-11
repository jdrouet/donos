pub mod header;
pub mod question;
pub mod record;

use crate::buffer::reader::ReaderError;
use crate::buffer::writer::WriterError;
use crate::buffer::BytePacketBuffer;

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

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DnsPacket {
    pub header: header::Header,
    pub questions: Vec<question::Question>,
    pub answers: Vec<record::Record>,
    pub authorities: Vec<record::Record>,
    pub resources: Vec<record::Record>,
}

impl DnsPacket {
    pub fn response_from(request: &Self) -> Self {
        Self {
            header: header::Header::response_from(&request.header),
            questions: request.questions.clone(),
            ..Default::default()
        }
    }

    pub fn new(header: header::Header) -> Self {
        Self {
            header,
            ..Default::default()
        }
    }

    pub fn with_question(mut self, question: question::Question) -> Self {
        self.questions.push(question);
        self
    }

    pub fn with_answer(mut self, record: record::Record) -> Self {
        self.answers.push(record);
        self
    }

    pub fn with_answers(mut self, records: Vec<record::Record>) -> Self {
        self.answers.extend(records);
        self
    }

    pub fn with_authority(mut self, record: record::Record) -> Self {
        self.authorities.push(record);
        self
    }

    pub fn with_resource(mut self, record: record::Record) -> Self {
        self.resources.push(record);
        self
    }
}

impl TryFrom<BytePacketBuffer> for DnsPacket {
    type Error = ReaderError;

    fn try_from(mut buffer: BytePacketBuffer) -> Result<Self, Self::Error> {
        let header = header::Header::read(&mut buffer)?;

        let question_count = buffer.read_u16()? as usize;
        let answer_count = buffer.read_u16()? as usize;
        let authority_count = buffer.read_u16()? as usize;
        let resource_count = buffer.read_u16()? as usize;

        let mut questions = Vec::with_capacity(question_count);
        for _ in 0..question_count {
            questions.push(question::Question::read(&mut buffer)?);
        }

        let mut answers = Vec::with_capacity(answer_count);
        for _ in 0..answer_count {
            answers.push(record::Record::read(&mut buffer)?);
        }

        let mut authorities = Vec::with_capacity(authority_count);
        for _ in 0..authority_count {
            authorities.push(record::Record::read(&mut buffer)?);
        }

        let mut resources = Vec::with_capacity(resource_count);
        for _ in 0..resource_count {
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
    pub fn create_buffer(&self) -> Result<BytePacketBuffer, WriterError> {
        let mut buffer = BytePacketBuffer::default();
        self.header.write(&mut buffer)?;

        buffer.write_u16(self.questions.len() as u16)?;
        buffer.write_u16(self.answers.len() as u16)?;
        buffer.write_u16(self.authorities.len() as u16)?;
        buffer.write_u16(self.resources.len() as u16)?;

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

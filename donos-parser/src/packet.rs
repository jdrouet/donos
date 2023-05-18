use crate::buffer::{BytePacketBuffer, ReaderError, WriterError};
use crate::{DnsHeader, DnsQuestion, DnsRecord};

#[derive(Clone, Debug, Default)]
pub struct DnsPacket {
    pub header: DnsHeader,
    pub questions: Vec<DnsQuestion>,
    pub answers: Vec<DnsRecord>,
    pub authorities: Vec<DnsRecord>,
    pub resources: Vec<DnsRecord>,
}

impl TryFrom<BytePacketBuffer> for DnsPacket {
    type Error = ReaderError;

    fn try_from(mut buffer: BytePacketBuffer) -> Result<Self, Self::Error> {
        let header = DnsHeader::read(&mut buffer)?;

        let mut questions = Vec::with_capacity(header.questions as usize);
        for _ in 0..header.questions {
            questions.push(DnsQuestion::read(&mut buffer)?);
        }

        let mut answers = Vec::with_capacity(header.answers as usize);
        for _ in 0..header.answers {
            answers.push(DnsRecord::read(&mut buffer)?);
        }

        let mut authorities = Vec::with_capacity(header.authoritative_entries as usize);
        for _ in 0..header.authoritative_entries {
            authorities.push(DnsRecord::read(&mut buffer)?);
        }

        let mut resources = Vec::with_capacity(header.resource_entries as usize);
        for _ in 0..header.resource_entries {
            resources.push(DnsRecord::read(&mut buffer)?);
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

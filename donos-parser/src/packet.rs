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

impl DnsPacket {
    pub fn from_buffer(buffer: &mut BytePacketBuffer) -> Result<DnsPacket, ReaderError> {
        let header = DnsHeader::read(buffer)?;

        let mut questions = Vec::with_capacity(header.questions as usize);
        for _ in 0..header.questions {
            questions.push(DnsQuestion::read(buffer)?);
        }

        let mut answers = Vec::with_capacity(header.answers as usize);
        for _ in 0..header.answers {
            answers.push(DnsRecord::read(buffer)?);
        }

        let mut authorities = Vec::with_capacity(header.authoritative_entries as usize);
        for _ in 0..header.authoritative_entries {
            authorities.push(DnsRecord::read(buffer)?);
        }

        let mut resources = Vec::with_capacity(header.resource_entries as usize);
        for _ in 0..header.resource_entries {
            resources.push(DnsRecord::read(buffer)?);
        }

        Ok(DnsPacket {
            header,
            questions,
            answers,
            authorities,
            resources,
        })
    }

    pub fn write(&mut self, buffer: &mut BytePacketBuffer) -> Result<(), WriterError> {
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

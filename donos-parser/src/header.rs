use crate::buffer::{BytePacketBuffer, ReaderError, WriterError};

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

impl Default for DnsHeader {
    fn default() -> Self {
        Self {
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
}

impl DnsHeader {
    pub fn read(buffer: &mut BytePacketBuffer) -> Result<Self, ReaderError> {
        let id = buffer.read_u16()?;

        let flags = buffer.read_u16()?;
        let head = (flags >> 8) as u8;
        let tail = (flags & 0xFF) as u8;

        let questions = buffer.read_u16()?;
        let answers = buffer.read_u16()?;
        let authoritative_entries = buffer.read_u16()?;
        let resource_entries = buffer.read_u16()?;

        Ok(Self {
            id,
            recursion_desired: (head & (1 << 0)) > 0,
            truncated_message: (head & (1 << 1)) > 0,
            authoritative_answer: (head & (1 << 2)) > 0,
            opcode: (head >> 3) & 0x0F,
            response: (head & (1 << 7)) > 0,
            response_code: ResponseCode::from_num(tail & 0x0F),
            checking_disabled: (tail & (1 << 4)) > 0,
            authed_data: (tail & (1 << 5)) > 0,
            z: (tail & (1 << 6)) > 0,
            recursion_available: (tail & (1 << 7)) > 0,
            questions,
            answers,
            authoritative_entries,
            resource_entries,
        })
    }

    pub fn write(&self, buffer: &mut BytePacketBuffer) -> Result<(), WriterError> {
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

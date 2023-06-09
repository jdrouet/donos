use crate::buffer::reader::ReaderError;
use crate::buffer::writer::WriterError;
use crate::buffer::BytePacketBuffer;

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

impl TryFrom<u8> for ResponseCode {
    type Error = ReaderError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ResponseCode::NoError),
            1 => Ok(ResponseCode::FormatError),
            2 => Ok(ResponseCode::ServerFailure),
            3 => Ok(ResponseCode::NameError),
            4 => Ok(ResponseCode::NotImplemented),
            5 => Ok(ResponseCode::Refused),
            other => Err(ReaderError::InvalidResponseCode(other)),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Header {
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
}

impl Header {
    pub fn question(id: u16) -> Self {
        Self {
            id,
            ..Default::default()
        }
    }

    pub fn response(id: u16) -> Self {
        Self {
            id,
            response: true,
            ..Default::default()
        }
    }

    pub fn response_from(request: &Self) -> Self {
        Self {
            id: request.id,
            recursion_desired: request.recursion_desired,
            truncated_message: false,
            authoritative_answer: false,
            opcode: request.opcode,
            response: true,
            response_code: ResponseCode::NoError,
            checking_disabled: false,
            authed_data: false,
            z: false,
            recursion_available: false,
        }
    }

    pub fn with_response_code(mut self, value: ResponseCode) -> Self {
        self.response_code = value;
        self
    }
}

impl Default for Header {
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
        }
    }
}

impl Header {
    /// Reads the first 4 bytes
    pub fn read(buffer: &mut BytePacketBuffer) -> Result<Self, ReaderError> {
        let id = buffer.read_u16()?;

        let head = buffer.read()?;
        let tail = buffer.read()?;

        Ok(Self {
            id,
            recursion_desired: (head & (1 << 0)) > 0,
            truncated_message: (head & (1 << 1)) > 0,
            authoritative_answer: (head & (1 << 2)) > 0,
            opcode: (head >> 3) & 0x0F,
            response: (head & (1 << 7)) > 0,
            response_code: ResponseCode::try_from(tail & 0x0F)?,
            checking_disabled: (tail & (1 << 4)) > 0,
            authed_data: (tail & (1 << 5)) > 0,
            z: (tail & (1 << 6)) > 0,
            recursion_available: (tail & (1 << 7)) > 0,
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

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    #[ignore = "only used when generating packets"]
    fn should_write_empty_packet() {
        let header = super::Header {
            id: 1,
            recursion_desired: true,
            truncated_message: false,
            authoritative_answer: true,
            opcode: 0,
            response: false,
            response_code: super::ResponseCode::NoError,
            checking_disabled: false,
            authed_data: false,
            z: false,
            recursion_available: true,
        };
        let mut buffer = crate::buffer::BytePacketBuffer::default();
        header.write(&mut buffer).unwrap();
        let buffer = buffer.buf;
        std::fs::write("data/only_header_query.bin", buffer).unwrap();
    }
}

use std::fmt::Display;

use super::BytePacketBuffer;

const MAX_JUMP: usize = 5;

#[derive(Debug, PartialEq, Eq)]
pub enum ReaderError {
    EndOfBuffer,
    TooManyJumps(usize),
    InvalidResponseCode(u8),
    InvalidClass(u16),
}

impl Display for ReaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EndOfBuffer => write!(f, "end of buffer"),
            Self::TooManyJumps(limit) => write!(f, "reached the limit of {limit} jumps"),
            Self::InvalidResponseCode(code) => write!(f, "invalid response code {code}"),
            Self::InvalidClass(code) => write!(f, "invalid class {code}"),
        }
    }
}

impl std::error::Error for ReaderError {}

impl From<ReaderError> for std::io::Error {
    fn from(value: ReaderError) -> Self {
        match value {
            ReaderError::EndOfBuffer => {
                std::io::Error::new(std::io::ErrorKind::InvalidData, "reading out of buffer")
            }
            ReaderError::TooManyJumps(size) => std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("too many jumps when reading: {size}"),
            ),
            ReaderError::InvalidResponseCode(value) => std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("invalid response code: {value}"),
            ),
            ReaderError::InvalidClass(value) => std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("invalid class: {value}"),
            ),
        }
    }
}

impl BytePacketBuffer {
    /// Step the buffer position forward a specific number of steps
    pub fn step(&mut self, steps: usize) -> Result<(), ReaderError> {
        self.pos += steps;

        Ok(())
    }

    /// Change the buffer position
    fn seek(&mut self, pos: usize) -> Result<(), ReaderError> {
        self.pos = pos;

        Ok(())
    }

    /// Read a single byte and move the position one step forward
    pub fn read(&mut self) -> Result<u8, ReaderError> {
        if self.pos >= 512 {
            return Err(ReaderError::EndOfBuffer);
        }
        let res = self.buf[self.pos];
        self.pos += 1;

        Ok(res)
    }

    /// Get a single byte, without changing the buffer position
    fn get(&self, pos: usize) -> Result<u8, ReaderError> {
        if pos >= 512 {
            return Err(ReaderError::EndOfBuffer);
        }
        Ok(self.buf[pos])
    }

    /// Get a range of bytes
    pub fn get_range(&self, start: usize, len: usize) -> Result<&[u8], ReaderError> {
        let end = start + len;
        if end >= 512 {
            return Err(ReaderError::EndOfBuffer);
        }
        Ok(&self.buf[start..end])
    }

    /// Read two bytes, stepping two steps forward
    pub fn read_u16(&mut self) -> Result<u16, ReaderError> {
        let res = ((self.read()? as u16) << 8) | (self.read()? as u16);

        Ok(res)
    }

    /// Read four bytes, stepping four steps forward
    pub fn read_u32(&mut self) -> Result<u32, ReaderError> {
        let res = ((self.read()? as u32) << 24)
            | ((self.read()? as u32) << 16)
            | ((self.read()? as u32) << 8)
            | (self.read()? as u32);

        Ok(res)
    }

    fn recursive_read_qname(
        &mut self,
        position: usize,
        jumps_count: usize,
    ) -> Result<(String, usize), ReaderError> {
        // Dns Packets are untrusted data, so we need to be paranoid.
        // Someone can craft a packet with a cycle in the jump instructions.
        // This guards against such packets.
        if jumps_count > MAX_JUMP {
            return Err(ReaderError::TooManyJumps(MAX_JUMP));
        }

        // At this point, we're always at the beginning of a label. Recall
        // that labels start with a length byte.
        let length = self.get(position)?;

        // If `length` has the two most significant bit are set, it represents a
        // jump to some other offset in the packet:
        if (length & 0xC0) == 0xC0 {
            // Read another byte, calculate offset and perform the jump by
            // updating our local position variable
            let b2 = self.get(position + 1)? as u16;
            let offset = ((((length as u16) ^ 0xC0) << 8) | b2) as usize;

            let label = if let Some(label) = self.reading_labels.get(&offset) {
                label.to_owned()
            } else {
                let (label, _) = self.recursive_read_qname(offset, jumps_count + 1)?;
                label
            };
            Ok((label, position + 2))
        } else if length == 0 {
            // Domain names are terminated by an empty label of length 0,
            // so if the length is zero we're done.
            Ok((String::new(), position + 1))
        } else {
            // The base scenario, where we're reading a single label and
            // appending it to the output
            let length = length as usize;
            // Extract the actual ASCII bytes for this label and append them
            // to the output buffer.
            let str_buffer = self.get_range(position + 1, length)?;
            let label = String::from_utf8_lossy(str_buffer).to_lowercase();

            let next_position = position + 1 + length;
            let (next_label, next_position) =
                self.recursive_read_qname(next_position, jumps_count)?;

            let label = if next_label.is_empty() {
                label
            } else {
                format!("{label}.{next_label}")
            };
            self.reading_labels.insert(position, label.clone());
            Ok((label, next_position))
        }
    }

    /// Read a qname
    ///
    /// The tricky part: Reading domain names, taking labels into consideration.
    /// Will take something like [3]www[6]google[3]com[0] and append
    /// www.google.com to outstr.
    pub fn read_qname(&mut self) -> Result<String, ReaderError> {
        let (label, position) = self.recursive_read_qname(self.pos(), 0)?;
        self.seek(position)?;
        Ok(label)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn should_read_empty_qname() {
        let mut buffer = crate::buffer::BytePacketBuffer::default();
        buffer.buf[0] = 0;
        let result = buffer.read_qname().unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn should_read_simple_qname() {
        let mut buffer = crate::buffer::BytePacketBuffer::default();
        buffer.buf[0] = 2;
        buffer.buf[1] = b'a';
        buffer.buf[2] = b'b';
        buffer.buf[3] = 0;
        let result = buffer.read_qname().unwrap();
        assert_eq!(result, "ab");
    }

    #[test]
    fn should_read_multiple_section_qname() {
        let mut buffer = crate::buffer::BytePacketBuffer::default();
        buffer.buf[0] = 2;
        buffer.buf[1] = b'a';
        buffer.buf[2] = b'b';
        buffer.buf[3] = 1;
        buffer.buf[4] = b'c';
        buffer.buf[5] = 1;
        buffer.buf[6] = b'd';
        let result = buffer.read_qname().unwrap();
        assert_eq!(result, "ab.c.d");
    }

    #[test]
    fn should_fail_read_qname_with_loop() {
        let mut buffer = crate::buffer::BytePacketBuffer::default();
        buffer.buf[0] = 2;
        buffer.buf[1] = b'a';
        buffer.buf[2] = b'b';
        buffer.buf[3] = 0xC0;
        let error = buffer.read_qname().unwrap_err();
        assert_eq!(error, super::ReaderError::TooManyJumps(5));
    }

    #[test]
    fn should_read_qname_with_redirect() {
        println!("{}", 0xC2);
        let mut buffer = crate::buffer::BytePacketBuffer::default();
        buffer.buf[0] = 1;
        buffer.buf[1] = b'b';
        buffer.buf[2] = 1;
        buffer.buf[3] = b'c';
        buffer.buf[4] = 0;
        buffer.buf[5] = 1;
        buffer.buf[6] = b'd';
        buffer.buf[7] = 0xC0;
        buffer.buf[8] = 2;
        buffer.pos = 5;
        let result = buffer.read_qname().unwrap();
        assert_eq!(result, "d.c");
    }
}

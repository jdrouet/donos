use super::BytePacketBuffer;

#[derive(Debug)]
pub enum WriterError {
    EndOfBuffer,
    SingleLabelLengh,
}

impl From<WriterError> for std::io::Error {
    fn from(value: WriterError) -> Self {
        match value {
            WriterError::EndOfBuffer => {
                std::io::Error::new(std::io::ErrorKind::InvalidData, "writing out of buffer")
            }
            WriterError::SingleLabelLengh => std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "single label too long when writing",
            ),
        }
    }
}

impl BytePacketBuffer {
    fn set(&mut self, pos: usize, val: u8) -> Result<(), WriterError> {
        self.buf[pos] = val;

        Ok(())
    }

    pub fn set_u16(&mut self, pos: usize, val: u16) -> Result<(), WriterError> {
        self.set(pos, (val >> 8) as u8)?;
        self.set(pos + 1, (val & 0xFF) as u8)?;

        Ok(())
    }

    fn write(&mut self, val: u8) -> Result<(), WriterError> {
        if self.pos >= 512 {
            return Err(WriterError::EndOfBuffer);
        }
        self.buf[self.pos] = val;
        self.pos += 1;
        Ok(())
    }

    pub fn write_u8(&mut self, val: u8) -> Result<(), WriterError> {
        self.write(val)?;

        Ok(())
    }

    pub fn write_u16(&mut self, val: u16) -> Result<(), WriterError> {
        self.write((val >> 8) as u8)?;
        self.write((val & 0xFF) as u8)?;

        Ok(())
    }

    pub fn write_u32(&mut self, val: u32) -> Result<(), WriterError> {
        self.write(((val >> 24) & 0xFF) as u8)?;
        self.write(((val >> 16) & 0xFF) as u8)?;
        self.write(((val >> 8) & 0xFF) as u8)?;
        self.write((val & 0xFF) as u8)?;

        Ok(())
    }

    fn write_label(&mut self, label: &str) -> Result<(), WriterError> {
        let len = label.len() as u8;
        if len > 0x3f {
            return Err(WriterError::SingleLabelLengh);
        }
        self.write_u8(len as u8)?;
        for b in label.as_bytes() {
            self.write_u8(*b)?;
        }
        Ok(())
    }

    fn recursive_write_qname(&mut self, qname: &str) -> Result<bool, WriterError> {
        if let Some(index) = self.writing_labels.get(qname) {
            self.write_u16(0xC000 | (*index as u16))?;
            Ok(true)
        } else {
            self.writing_labels.insert(qname.to_string(), self.pos());
            if let Some((head, tail)) = qname.split_once('.') {
                self.write_label(head)?;
                self.recursive_write_qname(tail)
            } else {
                self.write_label(qname)?;
                Ok(false)
            }
        }
    }

    pub fn write_qname(&mut self, qname: &str) -> Result<(), WriterError> {
        if !self.recursive_write_qname(qname)? {
            self.write_u8(0)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn should_write_empty_qname() {
        let mut buffer = crate::buffer::BytePacketBuffer::default();
        buffer.write_qname("").unwrap();
        assert_eq!(buffer.pos, 2);
        assert_eq!(buffer.buf[0], 0);
        assert_eq!(buffer.buf[1], 0);
    }

    #[test]
    fn should_write_simple_qname() {
        let mut buffer = crate::buffer::BytePacketBuffer::default();
        buffer.write_qname("www.foo.bar").unwrap();
        assert_eq!(buffer.buf[0], 3);
        assert_eq!(buffer.buf[1], b'w');
        assert_eq!(buffer.buf[2], b'w');
        assert_eq!(buffer.buf[3], b'w');
        assert_eq!(buffer.buf[4], 3);
        assert_eq!(buffer.buf[5], b'f');
        assert_eq!(buffer.buf[6], b'o');
        assert_eq!(buffer.buf[7], b'o');
        assert_eq!(buffer.buf[8], 3);
        assert_eq!(buffer.buf[9], b'b');
        assert_eq!(buffer.buf[10], b'a');
        assert_eq!(buffer.buf[11], b'r');
        assert_eq!(buffer.buf[12], 0);
        assert_eq!(buffer.pos, 13);
    }

    #[test]
    fn should_write_qname_with_redirect() {
        let mut buffer = crate::buffer::BytePacketBuffer::default();
        buffer.write_qname("www.foo.bar").unwrap();
        buffer.write_qname("what.foo.bar").unwrap();
        assert_eq!(buffer.buf[13], 4);
        assert_eq!(buffer.buf[14], b'w');
        assert_eq!(buffer.buf[15], b'h');
        assert_eq!(buffer.buf[16], b'a');
        assert_eq!(buffer.buf[17], b't');
        assert_eq!(buffer.buf[18], 0xC0);
        assert_eq!(buffer.buf[19], 0x04);
        assert_eq!(buffer.pos, 20);
    }
}

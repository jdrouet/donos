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

    pub fn write_qname(&mut self, qname: &str) -> Result<(), WriterError> {
        for label in qname.split('.') {
            let len = label.len();
            if len > 0x3f {
                return Err(WriterError::SingleLabelLengh);
            }

            self.write_u8(len as u8)?;
            for b in label.as_bytes() {
                self.write_u8(*b)?;
            }
        }

        self.write_u8(0)?;

        Ok(())
    }
}

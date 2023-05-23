use super::BytePacketBuffer;

#[derive(Debug)]
pub enum ReaderError {
    EndOfBuffer,
    TooManyJumps(usize),
    InvalidResponseCode(u8),
    InvalidClass(u16),
}

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
    fn get(&mut self, pos: usize) -> Result<u8, ReaderError> {
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

    /// Read a qname
    ///
    /// The tricky part: Reading domain names, taking labels into consideration.
    /// Will take something like [3]www[6]google[3]com[0] and append
    /// www.google.com to outstr.
    pub fn read_qname(&mut self) -> Result<String, ReaderError> {
        // Since we might encounter jumps, we'll keep track of our position
        // locally as opposed to using the position within the struct. This
        // allows us to move the shared position to a point past our current
        // qname, while keeping track of our progress on the current qname
        // using this variable.
        let mut pos = self.pos();

        // track whether or not we've jumped
        let mut jumped = false;
        let max_jumps = 5;
        let mut jumps_performed = 0;

        let mut sections: Vec<String> = Vec::new();

        loop {
            // Dns Packets are untrusted data, so we need to be paranoid. Someone
            // can craft a packet with a cycle in the jump instructions. This guards
            // against such packets.
            if jumps_performed > max_jumps {
                return Err(ReaderError::TooManyJumps(max_jumps));
            }

            // At this point, we're always at the beginning of a label. Recall
            // that labels start with a length byte.
            let len = self.get(pos)?;

            // If len has the two most significant bit are set, it represents a
            // jump to some other offset in the packet:
            if (len & 0xC0) == 0xC0 {
                // Update the buffer position to a point past the current
                // label. We don't need to touch it any further.
                if !jumped {
                    self.seek(pos + 2)?;
                }

                // Read another byte, calculate offset and perform the jump by
                // updating our local position variable
                let b2 = self.get(pos + 1)? as u16;
                let offset = (((len as u16) ^ 0xC0) << 8) | b2;
                pos = offset as usize;

                // Indicate that a jump was performed.
                jumped = true;
                jumps_performed += 1;

                continue;
            }
            // The base scenario, where we're reading a single label and
            // appending it to the output:
            else {
                let label_index = pos;
                // Move a single byte forward to move past the length byte.
                pos += 1;

                // Domain names are terminated by an empty label of length 0,
                // so if the length is zero we're done.
                if len == 0 {
                    break;
                }

                // Extract the actual ASCII bytes for this label and append them
                // to the output buffer.
                let str_buffer = self.get_range(pos, len as usize)?;
                let section = String::from_utf8_lossy(str_buffer).to_lowercase();
                self.labels.insert(label_index, section.clone());
                sections.push(section);

                // Move forward the full length of the label.
                pos += len as usize;
            }
        }

        if !jumped {
            self.seek(pos)?;
        }

        Ok(sections.join("."))
    }
}

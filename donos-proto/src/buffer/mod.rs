pub mod reader;
pub mod writer;

#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary, Debug, Clone))]
pub struct BytePacketBuffer {
    pub buf: [u8; 512],
    pub pos: usize,
}

impl Default for BytePacketBuffer {
    /// This gives us a fresh buffer for holding the packet contents, and a
    /// field for keeping track of where we are.
    fn default() -> Self {
        BytePacketBuffer {
            buf: [0; 512],
            pos: 0,
        }
    }
}

impl BytePacketBuffer {
    /// Current position within buffer
    pub fn pos(&self) -> usize {
        self.pos
    }
}

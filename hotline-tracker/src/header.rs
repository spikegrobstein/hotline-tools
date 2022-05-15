use bytes::{Buf, BytesMut, BufMut};

const MAGIC_WORD_LEN: usize = 4;
const MAGIC_WORD: &[u8; MAGIC_WORD_LEN] = b"HTRK";
const VERSION: u16 = 1;
const HEADER_LEN: usize = 6;

#[derive(Debug)]
pub struct Header {
    magic_word: [u8; MAGIC_WORD_LEN],
    version: u16,
}

impl Default for Header {
    fn default() -> Self {
        Self {
            magic_word: MAGIC_WORD.to_owned(),
            version: VERSION,
        }
    }
}

impl Header {
    pub fn is_valid(&self) -> bool {
        &self.magic_word == MAGIC_WORD && self.version == VERSION
    }

    pub fn from_bytes(mut bytes: &[u8]) -> Option<Self> {
        if bytes.remaining() < HEADER_LEN {
            return None;
        }

        // we can .unwrap() because we know we have enough bytes.
        let magic_word: [u8; MAGIC_WORD_LEN] = bytes[..MAGIC_WORD_LEN].try_into().unwrap();
        bytes.advance(MAGIC_WORD.len());
        let version = bytes.get_u16();

        Some(Self {
            magic_word,
            version,
        })
    }

    pub fn as_bytes(&self) -> BytesMut {
        let mut buf = BytesMut::with_capacity(HEADER_LEN);

        self.put_slice(&mut buf);

        buf
    }

    pub fn put_slice(&self, buf: &mut BytesMut) -> usize {
        buf.put_slice(&self.magic_word);
        buf.put_u16(self.version);

        HEADER_LEN
    }
}

use bytes::{Buf, BufMut, BytesMut};

#[derive(Debug)]
pub struct UpdateRecord {
    pub version: u16,
    pub remaining_data_size: u16,
    pub total_servers: u16,
    pub remaining_servers: u16,
}

impl UpdateRecord {
    pub fn from_bytes(mut bytes: &[u8]) -> Option<Self> {
        if bytes.remaining() < 8 {
            return None;
        }

        let version = bytes.get_u16();
        let remaining_data_size = bytes.get_u16();
        let total_servers = bytes.get_u16();
        let remaining_servers = bytes.get_u16();

        let update_record = Self {
            version,
            remaining_data_size,
            total_servers,
            remaining_servers,
        };

        Some(update_record)
    }

    pub fn data_size(&self) -> usize {
        8
    }

    pub fn put_slice(&self, buf: &mut BytesMut) -> usize {
        buf.put_u16(self.version);
        buf.put_u16(self.remaining_data_size);
        buf.put_u16(self.total_servers);
        buf.put_u16(self.remaining_servers);

        self.data_size()
    }

    pub fn as_bytes(&self) -> BytesMut {
        let mut buf = BytesMut::with_capacity(8);

        self.put_slice(&mut buf);

        buf
    }
}

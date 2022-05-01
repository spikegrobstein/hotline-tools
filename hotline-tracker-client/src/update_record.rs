use bytes::{Buf, BytesMut};

#[derive(Debug)]
pub struct UpdateRecord {
    pub version: u16,
    pub users_online: u16,
    pub total_servers: u16,
    pub unknown: u16,
}

impl UpdateRecord {
    pub fn from_bytes(bytes: &mut BytesMut) -> Option<Self> {
        if bytes.remaining() < 8 {
            return None;
        }

        let version = bytes.get_u16();
        let users_online = bytes.get_u16();
        let total_servers = bytes.get_u16();
        let unknown = bytes.get_u16();

        Some(Self {
            version,
            users_online,
            total_servers,
            unknown,
        })
    }
}

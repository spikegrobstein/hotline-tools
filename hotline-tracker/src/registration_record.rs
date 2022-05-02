use bytes::{Buf, BytesMut, BufMut};
use macroman_tools::macroman_to_string;

const REGISTRY_VERSION: u16 = 1;

/// the data that will be parsed from or encoded to a UDP packet
/// for a hotline server tracker registration.
///
/// Because we should always receive this packet as a single packet,
/// we only need to ensure that the packet is well-formed
///
/// TODO: this should return an error when parsing from bytes
#[derive(Debug, PartialEq)]
pub struct RegistrationRecord {
    pub port: u16,
    pub users_online: u16,
    pub reserved: u16,
    pub id: u32,
    pub name: String,
    pub description: String,
    pub password: String,
}

impl Default for RegistrationRecord {
    fn default() -> Self {
        Self {
            port: 5500,
            users_online: 0,
            reserved: 0,
            id: 0,
            name: "".into(),
            description: "".into(),
            password: "".into(),
        }
    }
}

impl RegistrationRecord {
    pub fn from_bytes(bytes: &mut BytesMut) -> Option<Self> {
        // these 15s are the static portion + length bytes (12 + 3)
        if bytes.remaining() < 15 {
            // not enough data for the absolute minimum size
            return None
        }

        let ex_name_len = bytes[12] as usize;
        if bytes.remaining() < 15 + ex_name_len {
            return None
        }

        let ex_desc_len = bytes[12 + 1 + ex_name_len] as usize;
        if bytes.remaining() < 15 + ex_name_len + ex_desc_len {
            return None
        }

        let ex_pass_len = bytes[12 + 1 + ex_name_len + 1 + ex_desc_len] as usize;
        if bytes.remaining() != 15 + ex_name_len + ex_desc_len + ex_pass_len {
            return None
        }

        let version = bytes.get_u16();

        assert_eq!(version, REGISTRY_VERSION);

        let port = bytes.get_u16();
        let users_online = bytes.get_u16();
        let reserved = bytes.get_u16();
        let id = bytes.get_u32();

        let name_len = bytes.get_u8() as usize;
        let name = macroman_to_string(&bytes[..name_len]);
        bytes.advance(name_len);

        let desc_len = bytes.get_u8() as usize;
        let description = macroman_to_string(&bytes[..desc_len]);
        bytes.advance(desc_len);

        let pass_len = bytes.get_u8() as usize;
        let password = macroman_to_string(&bytes[..pass_len]);
        bytes.advance(pass_len);

        Some(Self {
            port,
            users_online,
            reserved,
            id,
            name,
            description,
            password,
        })
    }

    pub fn to_bytes(&self) -> BytesMut {
        let mut buf = BytesMut::with_capacity(15 + self.name.len() + self.description.len() + self.password.len());

        buf.put_u16(REGISTRY_VERSION);
        buf.put_u16(self.port);
        buf.put_u16(self.users_online);
        buf.put_u16(self.reserved);
        buf.put_u32(self.id);

        buf.put_u8(self.name.len() as u8);
        buf.put_slice(self.name.as_bytes());

        buf.put_u8(self.description.len() as u8);
        buf.put_slice(self.description.as_bytes());

        buf.put_u8(self.password.len() as u8);
        buf.put_slice(self.password.as_bytes());

        buf
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses_back_and_forth() {
        let r = RegistrationRecord {
            name: "Test server".into(),
            description: "just a test".into(),
            id: 1234,
            ..Default::default()
        };

        let mut data = r.to_bytes();
        let new_r = RegistrationRecord::from_bytes(&mut data).unwrap();

        assert_eq!(r, new_r);
    }
}

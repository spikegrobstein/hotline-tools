use bytes::{Buf, BytesMut, BufMut};
use macroman_tools::MacRomanString;

use std::net::Ipv4Addr;

#[derive(Debug, Clone)]
pub struct ServerRecord {
    pub address: Ipv4Addr,
    pub port: u16,
    pub users_online: u16,
    pub reserved: u16,
    pub name: MacRomanString<255>,
    pub description: MacRomanString<255>,
}

impl Default for ServerRecord {
    fn default() -> Self {
        Self {
            address: Ipv4Addr::new(127, 0, 0, 1),
            port: 5500,
            users_online: 0,
            reserved: 0,
            name: "Hotline Server".into(),
            description: "".into(),
        }
    }
}

impl ServerRecord {
    pub fn from_bytes(mut bytes: &[u8]) -> Option<Self> {
        // first, let's make sure we have enough bytes in the buffer
        // to do this, we have to make sure we can read the name_len field
        // then that we have enough bytes to read that + desc_len + desc
        if bytes.remaining() < 12 { // name_len + 1 (desc_len)
            return None;
        }

        let ex_name_len: usize = bytes[10] as usize;
        if bytes.remaining() < 12 + ex_name_len {
            return None;
        }

        let ex_desc_len: usize = bytes[10 + 1 + ex_name_len] as usize;
        if bytes.remaining() < 12 + ex_name_len + ex_desc_len {
            // we know exactly how much we need for this next frame
            // so let's just reserve it.
            // bytes.reserve(12 + ex_name_len + ex_desc_len);

            return None;
        }

        // we have enough data, let's read the record.

        let address = Ipv4Addr::new(
            bytes.get_u8(),
            bytes.get_u8(),
            bytes.get_u8(),
            bytes.get_u8()
        );
        // eprintln!("address: {address}");
        let port = bytes.get_u16();
        // dbg!(port);
        let users_online = bytes.get_u16();
        // dbg!(users_online);
        let reserved = bytes.get_u16();
        // dbg!(reserved);
        let name_len = bytes.get_u8() as usize;
        // eprintln!("name_len: {name_len}");

        let name = bytes[..name_len].into();
        // dbg!(name);
        bytes.advance(name_len as usize);

        let desc_len = bytes.get_u8() as usize;
        let description = bytes[..desc_len].into();
        // dbg!(description);
        bytes.advance(desc_len as usize);

        let server_record = Self {
            address,
            port,
            users_online,
            reserved,
            name,
            description,
        };

        Some(server_record)
    }

    pub fn data_size(&self) -> usize {
        12 + self.name.len() + self.description.len()
    }

    pub fn as_bytes(&self) -> BytesMut {
        let mut buf = BytesMut::with_capacity(self.data_size());

        self.put_slice(&mut buf);

        buf
    }

    pub fn put_slice(&self, buf: &mut BytesMut) -> usize {
        let octets = self.address.octets();
        buf.put_u8(octets[0]);
        buf.put_u8(octets[1]);
        buf.put_u8(octets[2]);
        buf.put_u8(octets[3]);

        buf.put_u16(self.port);
        buf.put_u16(self.users_online);
        buf.put_u16(self.reserved);

        self.name.write_to_buf(buf);
        self.description.write_to_buf(buf);

        self.data_size()
    }
}

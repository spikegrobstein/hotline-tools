use bytes::{Buf, BytesMut};
use macroman_tools::macroman_to_string;

#[derive(Debug)]
pub struct ServerRecord {
    pub address: String,
    pub port: u16,
    pub users_online: u16,
    pub name: String,
    pub description: String,
}

impl ServerRecord {
    pub fn from_bytes(bytes: &mut BytesMut) -> Option<Self> {
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
            bytes.reserve(12 + ex_name_len + ex_desc_len);

            return None;
        }

        // we have enough data, let's read the record.

        let address = format!("{}.{}.{}.{}", bytes.get_u8(), bytes.get_u8(), bytes.get_u8(), bytes.get_u8());
        // eprintln!("address: {address}");
        let port = bytes.get_u16();
        // dbg!(port);
        let users_online = bytes.get_u16();
        // dbg!(users_online);
        let _reserved = bytes.get_u16();
        // dbg!(reserved);
        let name_len = bytes.get_u8() as usize;
        // eprintln!("name_len: {name_len}");

        let name = macroman_to_string(&bytes[..name_len]);
        // dbg!(name);
        bytes.advance(name_len as usize);

        let desc_len = bytes.get_u8() as usize;
        let description = macroman_to_string(&bytes[..desc_len]);
        // dbg!(description);
        bytes.advance(desc_len as usize);

        Some(Self {
            address,
            port,
            users_online,
            name,
            description,
        })
    }
}

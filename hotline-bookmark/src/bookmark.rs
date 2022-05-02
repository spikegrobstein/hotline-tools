use bytes::{BytesMut, Bytes, BufMut, Buf};
use macroman_tools::macroman_to_string;

use std::io::prelude::*;
use std::fs::File;

const BOOKMARK_MAGIC_WORD: &[u8; 4] = b"HTsc";
const BOOKMARK_VERSION: u16 = 1;
const BOOKMARK_HEADER_LEN: usize = 6;

const BOOKMARK_LENGTH: usize = 460; // a bookmark file is 460 bytes

// these are offsets to the length byte, which precedes each value
const USERNAME_OFFSET: usize = 135;
const PASSWORD_OFFSET: usize = 169;
const ADDRESS_OFFSET: usize = 203;

// header @ 0: magic word, version [0-5; 6]
// username @ 135: len, username [135-168; ]
// password @ 169: len, password [169-202]
// addres @ 203: len, address [203-459]
// padding
//

#[derive(Debug)]
pub struct Bookmark {
    pub address: String,
    pub username: String,
    pub password: String,
}

impl Bookmark {
    pub fn new(address: String) -> Self {
        Self {
            address,
            username: "".into(),
            password: "".into(),
        }
    }

    pub fn credentials(&mut self, username: String, password: String) -> &mut Self {
        // TODO: validate length. max len of these is 33.
        self.username = username;
        self.password = password;

        self
    }

    pub fn to_bytes(&self) -> BytesMut {
        let mut buf = BytesMut::with_capacity(BOOKMARK_LENGTH);

        // header
        buf.put_slice(BOOKMARK_MAGIC_WORD);
        buf.put_u16(BOOKMARK_VERSION);

        // zero padding
        buf.put_bytes(0, USERNAME_OFFSET - BOOKMARK_HEADER_LEN);

        // username
        buf.put_u8(self.username.len() as u8);
        buf.put_slice(self.username.as_bytes());
        buf.put_bytes(0, 33 - self.username.len());

        // password
        buf.put_u8(self.password.len() as u8);
        buf.put_slice(self.password.as_bytes());
        buf.put_bytes(0, 33 - self.password.len());

        // address
        buf.put_u8(self.address.len() as u8);
        buf.put_slice(self.address.as_bytes());
        buf.put_bytes(0, 256 - self.address.len());

        buf
    }

    pub fn from_file(path: &str) -> Result<Self, std::io::Error> {
        let data = std::fs::read(path)?;
        let mut buf = Bytes::from(data);

        dbg!(buf.len());

        // read header
        let magic_word: Vec<u8> = buf.copy_to_bytes(BOOKMARK_MAGIC_WORD.len()).to_vec();
        let version = buf.get_u16();

        assert_eq!(magic_word, BOOKMARK_MAGIC_WORD);
        assert_eq!(version, BOOKMARK_VERSION);

        buf.advance(USERNAME_OFFSET - BOOKMARK_HEADER_LEN);
        let username_len = buf.get_u8() as usize;
        let username: String = macroman_to_string(&buf.copy_to_bytes(username_len));

        buf.advance(33 - username_len);
        let password_len = buf.get_u8() as usize;
        let password: String = macroman_to_string(&buf.copy_to_bytes(password_len));

        buf.advance(33 - password_len);
        let address_len = buf.get_u8() as usize;
        let address: String = macroman_to_string(&buf.copy_to_bytes(address_len));

        Ok(Self {
            username,
            password,
            address,
        })
    }

    pub fn write_to_file(&self, path: &str) -> Result<usize, std::io::Error> {
        let mut f = File::create(path)?;
        let buf = self.to_bytes();

        f.write(&buf)
    }
}


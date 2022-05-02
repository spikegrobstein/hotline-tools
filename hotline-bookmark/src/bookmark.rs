use bytes::{BytesMut, Bytes, BufMut, Buf};

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
fn macroman_to_char(c: u8) -> char {
    // table lifted from: http://www.alanwood.net/demos/macroman.html
    //   Array.from($('.bord tbody').children).map((tr) => { const macno = tr.children[1].innerHTML; const unicode = tr.children[2].innerHTML; console.log(`(${macno}, '\\u{${parseInt(unicode).toString(16)}}')`)})
    let table: Vec<(u8, char)> = vec![
        (32, '\u{20}'),
        (33, '\u{21}'),
        (34, '\u{22}'),
        (35, '\u{23}'),
        (36, '\u{24}'),
        (37, '\u{25}'),
        (38, '\u{26}'),
        (39, '\u{27}'),
        (40, '\u{28}'),
        (41, '\u{29}'),
        (42, '\u{2a}'),
        (43, '\u{2b}'),
        (44, '\u{2c}'),
        (45, '\u{2d}'),
        (46, '\u{2e}'),
        (47, '\u{2f}'),
        (48, '\u{30}'),
        (49, '\u{31}'),
        (50, '\u{32}'),
        (51, '\u{33}'),
        (52, '\u{34}'),
        (53, '\u{35}'),
        (54, '\u{36}'),
        (55, '\u{37}'),
        (56, '\u{38}'),
        (57, '\u{39}'),
        (58, '\u{3a}'),
        (59, '\u{3b}'),
        (60, '\u{3c}'),
        (61, '\u{3d}'),
        (62, '\u{3e}'),
        (63, '\u{3f}'),
        (64, '\u{40}'),
        (65, '\u{41}'),
        (66, '\u{42}'),
        (67, '\u{43}'),
        (68, '\u{44}'),
        (69, '\u{45}'),
        (70, '\u{46}'),
        (71, '\u{47}'),
        (72, '\u{48}'),
        (73, '\u{49}'),
        (74, '\u{4a}'),
        (75, '\u{4b}'),
        (76, '\u{4c}'),
        (77, '\u{4d}'),
        (78, '\u{4e}'),
        (79, '\u{4f}'),
        (80, '\u{50}'),
        (81, '\u{51}'),
        (82, '\u{52}'),
        (83, '\u{53}'),
        (84, '\u{54}'),
        (85, '\u{55}'),
        (86, '\u{56}'),
        (87, '\u{57}'),
        (88, '\u{58}'),
        (89, '\u{59}'),
        (90, '\u{5a}'),
        (91, '\u{5b}'),
        (92, '\u{5c}'),
        (93, '\u{5d}'),
        (94, '\u{5e}'),
        (95, '\u{5f}'),
        (96, '\u{60}'),
        (97, '\u{61}'),
        (98, '\u{62}'),
        (99, '\u{63}'),
        (100, '\u{64}'),
        (101, '\u{65}'),
        (102, '\u{66}'),
        (103, '\u{67}'),
        (104, '\u{68}'),
        (105, '\u{69}'),
        (106, '\u{6a}'),
        (107, '\u{6b}'),
        (108, '\u{6c}'),
        (109, '\u{6d}'),
        (110, '\u{6e}'),
        (111, '\u{6f}'),
        (112, '\u{70}'),
        (113, '\u{71}'),
        (114, '\u{72}'),
        (115, '\u{73}'),
        (116, '\u{74}'),
        (117, '\u{75}'),
        (118, '\u{76}'),
        (119, '\u{77}'),
        (120, '\u{78}'),
        (121, '\u{79}'),
        (122, '\u{7a}'),
        (123, '\u{7b}'),
        (124, '\u{7c}'),
        (125, '\u{7d}'),
        (126, '\u{7e}'),
        (127, '\u{7f}'),
        (128, '\u{c4}'),
        (129, '\u{c5}'),
        (130, '\u{c7}'),
        (131, '\u{c9}'),
        (132, '\u{d1}'),
        (133, '\u{d6}'),
        (134, '\u{dc}'),
        (135, '\u{e1}'),
        (136, '\u{e0}'),
        (137, '\u{e2}'),
        (138, '\u{e4}'),
        (139, '\u{e3}'),
        (140, '\u{e5}'),
        (141, '\u{e7}'),
        (142, '\u{e9}'),
        (143, '\u{e8}'),
        (144, '\u{ea}'),
        (145, '\u{eb}'),
        (146, '\u{ed}'),
        (147, '\u{ec}'),
        (148, '\u{ee}'),
        (149, '\u{ef}'),
        (150, '\u{f1}'),
        (151, '\u{f3}'),
        (152, '\u{f2}'),
        (153, '\u{f4}'),
        (154, '\u{f6}'),
        (155, '\u{f5}'),
        (156, '\u{fa}'),
        (157, '\u{f9}'),
        (158, '\u{fb}'),
        (159, '\u{fc}'),
        (160, '\u{2020}'),
        (161, '\u{b0}'),
        (162, '\u{a2}'),
        (163, '\u{a3}'),
        (164, '\u{a7}'),
        (165, '\u{2022}'),
        (166, '\u{b6}'),
        (167, '\u{df}'),
        (168, '\u{ae}'),
        (169, '\u{a9}'),
        (170, '\u{2122}'),
        (171, '\u{b4}'),
        (172, '\u{a8}'),
        (173, '\u{2260}'),
        (174, '\u{c6}'),
        (175, '\u{d8}'),
        (176, '\u{221e}'),
        (177, '\u{b1}'),
        (178, '\u{2264}'),
        (179, '\u{2265}'),
        (180, '\u{a5}'),
        (181, '\u{b5}'),
        (182, '\u{2202}'),
        (183, '\u{2211}'),
        (184, '\u{220f}'),
        (185, '\u{3c0}'),
        (186, '\u{222b}'),
        (187, '\u{aa}'),
        (188, '\u{ba}'),
        (189, '\u{3a9}'),
        (190, '\u{e6}'),
        (191, '\u{f8}'),
        (192, '\u{bf}'),
        (193, '\u{a1}'),
        (194, '\u{ac}'),
        (195, '\u{221a}'),
        (196, '\u{192}'),
        (197, '\u{2248}'),
        (198, '\u{2206}'),
        (199, '\u{ab}'),
        (200, '\u{bb}'),
        (201, '\u{2026}'),
        (202, '\u{a0}'),
        (203, '\u{c0}'),
        (204, '\u{c3}'),
        (205, '\u{d5}'),
        (206, '\u{152}'),
        (207, '\u{153}'),
        (208, '\u{2013}'),
        (209, '\u{2014}'),
        (210, '\u{201c}'),
        (211, '\u{201d}'),
        (212, '\u{2018}'),
        (213, '\u{2019}'),
        (214, '\u{f7}'),
        (215, '\u{25ca}'),
        (216, '\u{ff}'),
        (217, '\u{178}'),
        (218, '\u{2044}'),
        (219, '\u{20ac}'),
        (220, '\u{2039}'),
        (221, '\u{203a}'),
        (222, '\u{fb01}'),
        (223, '\u{fb02}'),
        (224, '\u{2021}'),
        (225, '\u{b7}'),
        (226, '\u{201a}'),
        (227, '\u{201e}'),
        (228, '\u{2030}'),
        (229, '\u{c2}'),
        (230, '\u{ca}'),
        (231, '\u{c1}'),
        (232, '\u{cb}'),
        (233, '\u{c8}'),
        (234, '\u{cd}'),
        (235, '\u{ce}'),
        (236, '\u{cf}'),
        (237, '\u{cc}'),
        (238, '\u{d3}'),
        (239, '\u{d4}'),
        (240, '\u{f8ff}'),
        (241, '\u{d2}'),
        (242, '\u{da}'),
        (243, '\u{db}'),
        (244, '\u{d9}'),
        (245, '\u{131}'),
        (246, '\u{2c6}'),
        (247, '\u{2dc}'),
        (248, '\u{af}'),
        (249, '\u{2d8}'),
        (250, '\u{2d9}'),
        (251, '\u{2da}'),
        (252, '\u{b8}'),
        (253, '\u{2dd}'),
        (254, '\u{2db}'),
        (255, '\u{2c7}'),
    ];

    table.iter()
        .find(|(code, _)| *code == c)
        .map(|(_, unicode)| *unicode)
        .unwrap_or(c as char)
}

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
        let username: String = buf.copy_to_bytes(username_len).iter().map(|c| macroman_to_char(*c)).collect();

        buf.advance(33 - username_len);
        let password_len = buf.get_u8() as usize;
        let password: String = buf.copy_to_bytes(password_len).iter().map(|c| macroman_to_char(*c)).collect();

        buf.advance(33 - password_len);
        let address_len = buf.get_u8() as usize;
        let address: String = buf.copy_to_bytes(address_len).iter().map(|c| macroman_to_char(*c)).collect();

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


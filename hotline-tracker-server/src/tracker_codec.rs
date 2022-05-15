use tokio_util::codec::{Decoder, Encoder};
use hotline_tracker::{TrackerPacket, Header, UpdateRecord, ServerRecord};

use bytes::BytesMut;

#[derive(PartialEq)]
enum State {
    Initialized,
    ReceivedHeader,
}

pub struct TrackerCodec {
    state: State,
}

impl TrackerCodec {
    pub fn new() -> Self {
        Self {
            state: State::Initialized,
        }
    }
}

impl Decoder for TrackerCodec {
    type Item = TrackerPacket;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if self.state == State::Initialized {
            if let Some(header) = Header::from_bytes(&src) {
                if header.is_valid() {
                    self.state = State::ReceivedHeader;
                    return Ok(Some(TrackerPacket::Header))
                }

                panic!("bad header: {:?} / {}", header.magic_word, header.version);
            }

            panic!("failed to get header");
        }

        panic!("got data for some reason.");
    }
}

impl Encoder<TrackerPacket> for TrackerCodec {
    type Error = std::io::Error;

    fn encode(&mut self, pkt: TrackerPacket, dst: &mut BytesMut) -> Result<(), Self::Error> {
        match pkt {
            TrackerPacket::Header => {
                let header = Header::default();
                header.put_slice(dst);
            },
            TrackerPacket::Update(update) => {
                update.put_slice(dst);
            },
            TrackerPacket::Server(server) => {
                server.put_slice(dst);
            },
            TrackerPacket::Complete => {}, // no-op
        }

        Ok(())
    }
}


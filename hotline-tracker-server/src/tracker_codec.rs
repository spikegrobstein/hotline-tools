use tokio_util::codec::{Decoder, Encoder};
use hotline_tracker::{UpdateRecord, ServerRecord};

use bytes::{BytesMut, Buf, BufMut};

const HEADER: &[u8; 6] = b"HTRK\x00\x01";

#[derive(Debug, PartialEq)]
enum State {
    Initialized,
    ReceivedHeader,
    SentUpdate,
    Done,
}

pub enum TrackerPacket {
    Header,
    Update(UpdateRecord),
    Server(ServerRecord),
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
            let resp_header = &src[..6];
            if resp_header == HEADER {
                // got header
                self.state = State::ReceivedHeader;
                src.advance(6);
                return Ok(Some(TrackerPacket::Header))
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
                dst.put_slice(HEADER);
            },
            TrackerPacket::Update(update) => {
                update.put_slice(dst);
            },
            TrackerPacket::Server(server) => {
                server.put_slice(dst);
            },
        }

        Ok(())
    }
}


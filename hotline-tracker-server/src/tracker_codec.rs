use hotline_tracker::{Header, TrackerPacket};
use tokio_util::codec::{Decoder, Encoder};

use bytes::BytesMut;

use thiserror::Error;

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

#[derive(Debug, Error)]
pub enum CodecError {
    #[error("Invalid header: {0:?}/{1}")]
    InvalidHeader([u8; 4], u16),

    #[error("Failed to get header data")]
    NoHeader,

    #[error("Received unexpected data")]
    UnexpectedData,

    #[error("IO Error")]
    IoError(#[from] std::io::Error),
}

impl Decoder for TrackerCodec {
    type Item = TrackerPacket;
    type Error = CodecError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if self.state == State::Initialized {
            if let Some(header) = Header::from_bytes(src) {
                if header.is_valid() {
                    self.state = State::ReceivedHeader;
                    return Ok(Some(TrackerPacket::Header));
                }

                return Err(CodecError::InvalidHeader(header.magic_word, header.version));
            }

            return Err(CodecError::NoHeader);
        }

        Err(CodecError::UnexpectedData)
    }
}

impl Encoder<TrackerPacket> for TrackerCodec {
    type Error = std::io::Error;

    fn encode(&mut self, pkt: TrackerPacket, dst: &mut BytesMut) -> Result<(), Self::Error> {
        match pkt {
            TrackerPacket::Header => {
                let header = Header::default();
                header.put_slice(dst);
            }
            TrackerPacket::Update(update) => {
                update.put_slice(dst);
            }
            TrackerPacket::Server(server) => {
                server.put_slice(dst);
            }
            TrackerPacket::Complete => {} // no-op
        }

        Ok(())
    }
}

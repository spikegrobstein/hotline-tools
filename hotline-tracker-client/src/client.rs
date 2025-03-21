use bytes::{Buf, BytesMut};
use tokio::net::TcpStream;
use tokio_util::codec::Decoder;
use tokio_util::codec::Framed;
// use tokio_stream::StreamExt;
use tokio::io::AsyncWriteExt;

use hotline_tracker::header;
use hotline_tracker::{Header, ServerRecord, TrackerPacket, UpdateRecord};

// establish connection
// send HELO packet
// receive HELO reply
// receive a stream of server records and update records
// server closes connection

pub struct Client {
    pub framed_stream: Framed<TcpStream, HLTrackerCodec>,
}

impl Client {
    pub async fn connect(address: &str, port: u16) -> Result<Self, Box<dyn std::error::Error>> {
        // eprintln!("connecting... to {address}:{port}");

        let address = format!("{address}:{port}");
        let mut stream = TcpStream::connect(address).await?;

        let mut buf = BytesMut::with_capacity(6);
        Header::default().put_slice(&mut buf);
        stream.write_all(&buf).await?;

        let codec = HLTrackerCodec::new();
        let framed_stream = Framed::new(stream, codec);

        // eprintln!("initialized.");

        Ok(Self { framed_stream })
    }
}

#[derive(PartialEq, Eq)]
pub enum State {
    Initialized,
    ReceivedHeader,
}

pub struct HLTrackerCodec {
    state: State,
    expected_total_servers: Option<u16>,
    received_server_count: u16,
}

impl HLTrackerCodec {
    pub fn new() -> Self {
        Self {
            state: State::Initialized,
            expected_total_servers: None,
            received_server_count: 0,
        }
    }
}

impl Decoder for HLTrackerCodec {
    type Item = TrackerPacket;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if self.state == State::Initialized {
            if let Some(header) = Header::from_bytes(src) {
                if header.is_valid() {
                    src.advance(header::HEADER_LEN);
                    self.state = State::ReceivedHeader;
                    return Ok(Some(TrackerPacket::Header));
                } else {
                    panic!(
                        "invalid header {:?} / {}",
                        header.magic_word, header.version
                    );
                }
            }

            panic!("failed to get header.");
        }

        if let Some(expected_total) = self.expected_total_servers {
            if expected_total == self.received_server_count {
                return Ok(Some(TrackerPacket::Complete));
            }
        }

        if src.remaining() == 0 {
            return Ok(None);
        }

        // peek at the first byte
        // if it's a 0, it's the start to an update record
        // otherwise, it's the start to a server record
        if src[0] == 0 {
            // update packet
            // these are exactly 8 bytes so return early if not enough in buffer
            let update_record = UpdateRecord::from_bytes(src).map(TrackerPacket::Update);
            // dbg!(&update_record);
            if let Some(TrackerPacket::Update(ref update)) = update_record {
                src.advance(update.data_size());
                self.expected_total_servers = Some(update.total_servers);
            }

            Ok(update_record)
        } else {
            // server record
            let server_record =
                ServerRecord::from_bytes(src).map(|s| TrackerPacket::Server(s.into()));

            if let Some(TrackerPacket::Server(ref server_record)) = server_record {
                src.advance(server_record.data_size());
                self.received_server_count += 1;
            }

            // dbg!(&server_record);
            Ok(server_record)
        }
    }
}

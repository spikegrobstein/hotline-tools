use tokio::net::TcpListener;
use std::sync::{Mutex, Arc};

use crate::server_registry::ServerRegistry;
use crate::tracker_codec::TrackerCodec;
use hotline_tracker::TrackerPacket;

use tokio_util::codec::Framed;
use futures::{StreamExt, SinkExt};

pub const TRACKER_LISTEN_PORT: u16 = 5498;

pub struct TrackerListener {
    registry: Arc<Mutex<ServerRegistry>>,
}

impl TrackerListener {
    pub async fn listen(addr: &str, registry: Arc<Mutex<ServerRegistry>>) -> Result<(), Box<dyn std::error::Error>> {
        let socket = TcpListener::bind(addr).await?;

        loop {
            let (socket, addr) = socket.accept().await?;

            let registry = registry.clone();

            tokio::spawn(async move {
                let codec = TrackerCodec::new();
                let mut framed_stream = Framed::new(socket, codec);

                eprintln!("got a connection from {addr}");

                if let Ok(_) = framed_stream.next().await.unwrap() {
                    let (update, servers) = {
                        let mut registry = registry.lock().unwrap();
                        println!("got header.");
                        let update = registry.create_update_record();

                        let servers = registry.server_records();

                        (update, servers)
                    };

                    eprintln!("sending header and update");
                    framed_stream.send(TrackerPacket::Header).await.unwrap();
                    framed_stream.send(TrackerPacket::Update(update)).await.unwrap();

                    // TODO: this is probably fine for the scale we're at today, but this should
                    // emit updates in chunks.
                    for s in servers {
                        eprintln!("sending server record");
                        framed_stream.send(TrackerPacket::Server(s.into())).await.unwrap();
                    }
                }
            });
        }


    }
}

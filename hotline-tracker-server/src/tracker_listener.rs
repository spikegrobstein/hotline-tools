use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;

use crate::server_registry::ServerRegistry;
use crate::tracker_codec::TrackerCodec;
use hotline_tracker::TrackerPacket;

use futures::{SinkExt, StreamExt};
use tokio_util::codec::Framed;

use log::{debug, info};

pub struct TrackerListener {
    socket: TcpListener,
    registry: Arc<Mutex<ServerRegistry>>,
}

impl TrackerListener {
    pub const TRACKER_LISTEN_PORT: u16 = 5498;

    pub async fn new(
        addr: &str,
        port: u16,
        registry: Arc<Mutex<ServerRegistry>>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let interface = addr.parse::<IpAddr>()?;
        let sockaddr = SocketAddr::new(interface, port);
        let socket = TcpListener::bind(sockaddr).await?;

        Ok(Self { socket, registry })
    }

    pub async fn listen(&self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            let (socket, addr) = self.socket.accept().await?;

            let registry = self.registry.clone();

            tokio::spawn(async move {
                let codec = TrackerCodec::new();
                let mut framed_stream = Framed::new(socket, codec);

                info!("got a connection from {addr}");

                if framed_stream.next().await.unwrap().is_ok() {
                    let (update, servers) = {
                        let mut registry = registry.lock().unwrap();
                        debug!("got header.");
                        let update = registry.create_update_record();

                        let servers = registry.server_records();

                        (update, servers)
                    };

                    debug!("sending header and update");
                    framed_stream.send(TrackerPacket::Header).await.unwrap();
                    framed_stream
                        .send(TrackerPacket::Update(update))
                        .await
                        .unwrap();

                    // TODO: this is probably fine for the scale we're at today, but this should
                    // emit updates in chunks.
                    for s in servers {
                        debug!("sending server record");
                        framed_stream
                            .send(TrackerPacket::Server(s.into()))
                            .await
                            .unwrap();
                    }
                }
            });
        }
    }
}

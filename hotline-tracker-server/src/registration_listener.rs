use tokio::net::UdpSocket;
use tokio::sync::mpsc::Sender;
use std::net::{SocketAddr, IpAddr, Ipv4Addr};

use hotline_tracker::RegistrationRecord;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum RegistrationListenerError {
    #[error("No support for IPv6")]
    UnsupportedProtocol,
}

pub struct RegistrationListener {
    socket: UdpSocket,
    buf: [u8; 780],
    sender: Sender<(Ipv4Addr, RegistrationRecord)>,
}

impl RegistrationListener {
    pub const REGISTRATION_LISTEN_PORT: u16 = 5499;

    pub async fn new(addr: &str, port: u16, sender: Sender<(Ipv4Addr, RegistrationRecord)>) -> Result<Self, Box<dyn std::error::Error>> {
        let interface = addr.parse::<IpAddr>()?;
        let sockaddr = SocketAddr::new(interface, port);

        let socket = UdpSocket::bind(sockaddr).await?;

        Ok(Self {
            socket,
            buf: [0; 780],
            sender,
        })
    }

    pub async fn listen(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            let (len, addr) = self.socket.recv_from(&mut self.buf).await?;

            // TODO: have this return an Err if None.
            let r = RegistrationRecord::from_bytes(&self.buf[..len]).unwrap();

            if let SocketAddr::V4(addr) = addr {
                self.sender.send((*addr.ip(), r)).await?;
            } else {
                return Err(Box::new(RegistrationListenerError::UnsupportedProtocol))
            }
        }
    }
}

use tokio::net::UdpSocket;
use std::net::{SocketAddr, Ipv4Addr};

use hotline_tracker::RegistrationRecord;

pub const REGISTRATION_LISTEN_PORT: u16 = 5499;

pub struct RegistrationListener {
    socket: UdpSocket,
    buf: [u8; 780],
}

impl RegistrationListener {
    pub async fn listen(addr: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let socket = UdpSocket::bind(addr).await?;

        Ok(Self {
            socket,
            buf: [0; 780],
        })
    }

    pub async fn next_registration(&mut self) -> Result<(Ipv4Addr, RegistrationRecord), Box<dyn std::error::Error>> {
        let (len, addr) = self.socket.recv_from(&mut self.buf).await?;

        // TODO: have this return an Err if None.
        let r = RegistrationRecord::from_bytes(&self.buf[..len]).unwrap();

        if let SocketAddr::V4(addr) = addr {
            Ok((*addr.ip(), r))
        } else {
            // TODO: this should return an Err
            panic!("no support for ipv6")
        }
    }
}

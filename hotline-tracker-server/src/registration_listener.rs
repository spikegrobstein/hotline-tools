use tokio::net::UdpSocket;
use std::net::SocketAddr;

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

    pub async fn next_registration(&mut self) -> Result<(SocketAddr, RegistrationRecord), Box<dyn std::error::Error>> {
        let (len, addr) = self.socket.recv_from(&mut self.buf).await?;

        let r = RegistrationRecord::from_bytes(&self.buf[..len]).unwrap();

        Ok((addr, r))
    }
}

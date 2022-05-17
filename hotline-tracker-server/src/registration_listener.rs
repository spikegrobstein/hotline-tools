use tokio::net::UdpSocket;
use tokio::sync::mpsc::Sender;
use std::net::{SocketAddr, IpAddr, Ipv4Addr};

use hotline_tracker::RegistrationRecord;

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
                // TODO: this should return an Err
                panic!("no support for ipv6")
            }
        }
    }
}

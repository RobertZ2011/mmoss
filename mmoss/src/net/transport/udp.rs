use std::net::SocketAddr;

use crate::net::transport::{Addressed, AddressedFactory, Message, MessageFactory, Unreliable};
use anyhow::Result;
use async_trait::async_trait;
use tokio::net::{ToSocketAddrs, UdpSocket};

pub struct Udp<F: MessageFactory> {
    socket: UdpSocket,
    factory: AddressedFactory<SocketAddr, F>,
}

impl<F: MessageFactory> Udp<F> {
    pub async fn bind<A: ToSocketAddrs>(addr: A, factory: F) -> Result<Self> {
        Ok(Self {
            socket: UdpSocket::bind(addr).await?,
            factory: AddressedFactory::new(factory),
        })
    }
}

#[async_trait]
impl<F: MessageFactory> Unreliable<Addressed<SocketAddr, F::Message>> for Udp<F> {
    async fn send(&mut self, message: &Addressed<SocketAddr, F::Message>) -> Result<()> {
        let mut buffer = [0u8; 512];
        let len = message.serialize(&mut buffer)?;
        self.socket.send_to(&buffer[..len], message.address).await?;
        Ok(())
    }

    async fn receive(&mut self) -> Result<Addressed<SocketAddr, F::Message>> {
        let mut buffer = [0u8; 512];

        let (len, addr) = self.socket.recv_from(&mut buffer).await?;
        let (message, _) = self.factory.deserialize(&addr, &buffer[..len])?;
        Ok(message)
    }

    fn try_receive(&mut self) -> Result<Option<Addressed<SocketAddr, F::Message>>> {
        let mut buffer = [0u8; 512];

        match self.socket.try_recv_from(&mut buffer) {
            Ok((len, addr)) => {
                let (message, _) = self.factory.deserialize(&addr, &buffer[..len])?;
                Ok(Some(message))
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}

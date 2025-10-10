use std::{io::Cursor, net::SocketAddr};

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

#[async_trait(?Send)]
impl<F: MessageFactory> Unreliable<Addressed<SocketAddr, F::Message>> for Udp<F> {
    async fn send(&mut self, message: Addressed<SocketAddr, F::Message>) -> Result<()> {
        let mut buffer = [0u8; 512];
        let mut cursor = Cursor::new(buffer.as_mut_slice());

        message.serialize(&mut cursor)?;
        let len = cursor.position() as usize;
        self.socket.send_to(&buffer[..len], message.address).await?;
        Ok(())
    }

    async fn receive(&mut self) -> Result<Addressed<SocketAddr, F::Message>> {
        let mut buffer = [0u8; 512];

        let (len, addr) = self.socket.recv_from(&mut buffer).await?;
        let mut cursor = Cursor::new(&buffer[..len]);
        self.factory.deserialize(&addr, &mut cursor)
    }

    fn try_receive(&mut self) -> Result<Option<Addressed<SocketAddr, F::Message>>> {
        let mut buffer = [0u8; 512];

        match self.socket.try_recv_from(&mut buffer) {
            Ok((len, addr)) => {
                let mut cursor = Cursor::new(&buffer[..len]);
                Ok(Some(self.factory.deserialize(&addr, &mut cursor)?))
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}

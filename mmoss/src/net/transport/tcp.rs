use std::{io::Cursor, marker::PhantomData, net::SocketAddr};

use anyhow::Result;
use async_trait::async_trait;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream, ToSocketAddrs},
};

use crate::net::transport::{Message, MessageFactory, Unreliable};

pub struct Listener<M: Message> {
    listener: TcpListener,
    _marker: PhantomData<M>,
}

impl<M: Message> Listener<M> {
    pub async fn bind(addr: impl ToSocketAddrs) -> Result<Self> {
        Ok(Self {
            listener: TcpListener::bind(addr).await?,
            _marker: PhantomData,
        })
    }
    pub async fn accept<F: MessageFactory<Message = M>>(
        &self,
        factory: F,
    ) -> Result<(Connection<F>, SocketAddr)> {
        let (stream, addr) = self.listener.accept().await?;
        Ok((Connection { stream, factory }, addr))
    }
}

pub struct Connection<F: MessageFactory> {
    stream: TcpStream,
    factory: F,
}

impl<F: MessageFactory> Connection<F> {
    pub async fn connect(addr: impl ToSocketAddrs, factory: F) -> Result<Self> {
        let stream = TcpStream::connect(addr).await?;
        Ok(Self { stream, factory })
    }
}

#[async_trait(?Send)]
impl<F: MessageFactory> Unreliable<F::Message> for Connection<F> {
    async fn send(&mut self, message: F::Message) -> Result<()> {
        let mut buffer = [0u8; 512];
        let mut cursor = Cursor::new(buffer.as_mut_slice());

        message.serialize(&mut cursor)?;
        let len = cursor.position() as usize;
        self.stream.write(&buffer[..len]).await?;
        Ok(())
    }

    async fn receive(&mut self) -> Result<F::Message> {
        let mut buffer = [0u8; 512];
        let len = self.stream.read(&mut buffer).await?;
        let mut cursor = Cursor::new(&buffer[..len]);
        self.factory.deserialize(&(), &mut cursor)
    }

    fn try_receive(&mut self) -> Result<Option<F::Message>> {
        let mut buffer = [0u8; 512];
        match self.stream.try_read(&mut buffer) {
            Ok(len) if len > 0 => {
                let mut cursor = Cursor::new(&buffer[..len]);
                Ok(Some(self.factory.deserialize(&(), &mut cursor)?))
            }
            Ok(_) => Ok(None),
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}

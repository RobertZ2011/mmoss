use std::{collections::VecDeque, marker::PhantomData, mem::size_of, net::SocketAddr};

use anyhow::Result;
use async_trait::async_trait;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream, ToSocketAddrs},
};

use crate::net::transport::{Message, MessageFactory, Reliable, Unreliable};

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
        Ok((
            Connection {
                stream,
                factory,
                incoming: VecDeque::new(),
                receive_buffer: Vec::with_capacity(1024),
            },
            addr,
        ))
    }
}

pub struct Connection<F: MessageFactory> {
    stream: TcpStream,
    incoming: VecDeque<F::Message>,
    /// Buffer to contain a partial message read from the stream
    receive_buffer: Vec<u8>,
    factory: F,
}

impl<F: MessageFactory> Connection<F> {
    pub async fn connect(addr: impl ToSocketAddrs, factory: F) -> Result<Self> {
        let stream = TcpStream::connect(addr).await?;
        Ok(Self {
            stream,
            factory,
            incoming: VecDeque::new(),
            receive_buffer: Vec::with_capacity(1024),
        })
    }

    async fn receive_to_buffer(&mut self, target_len: usize) -> Result<usize> {
        while self.receive_buffer.len() < target_len {
            let mut buffer = [0u8; BUFFER_SIZE];
            let len = self.stream.read(&mut buffer).await?;
            self.receive_buffer.extend_from_slice(&buffer[..len]);
        }
        Ok(self.receive_buffer.len())
    }
}

const BUFFER_SIZE: usize = 512;

#[async_trait]
impl<F: MessageFactory> Unreliable<F::Message> for Connection<F> {
    async fn send(&mut self, message: &F::Message) -> Result<()> {
        let mut buffer = [0u8; BUFFER_SIZE];
        let len = message.serialize(&mut buffer[size_of::<u16>()..])?;
        let size_bytes = (len as u16).to_le_bytes();
        buffer[0] = size_bytes[0];
        buffer[1] = size_bytes[1];
        self.stream.write(&buffer[..len + size_of::<u16>()]).await?;
        Ok(())
    }

    async fn receive(&mut self) -> Result<F::Message> {
        if let Some(message) = self.incoming.pop_front() {
            Ok(message)
        } else {
            // Read at least one message
            let _ = self.receive_to_buffer(size_of::<u16>()).await?;
            let data_len =
                u16::from_le_bytes([self.receive_buffer[0], self.receive_buffer[1]]) as usize;
            let _ = self.receive_to_buffer(data_len).await?;

            // Deserialize messages
            while self.receive_buffer.len() >= size_of::<u16>() {
                let data_len =
                    u16::from_le_bytes([self.receive_buffer[0], self.receive_buffer[1]]) as usize;
                if self.receive_buffer[size_of::<u16>()..].len() < data_len {
                    break;
                }

                let data = self
                    .receive_buffer
                    .drain(..size_of::<u16>() + data_len)
                    .collect::<Vec<u8>>();
                let (message, _) = self.factory.deserialize(&(), &data[size_of::<u16>()..])?;
                self.incoming.push_back(message);
            }

            // Safe because the above ensures there is at least one message
            Ok(self.incoming.pop_front().unwrap())
        }
    }

    fn try_receive(&mut self) -> Result<Option<F::Message>> {
        let mut buffer = [0u8; BUFFER_SIZE];
        match self.stream.try_read(&mut buffer) {
            Ok(len) if len > 0 => {
                let (message, _) = self.factory.deserialize(&(), &buffer[..len])?;
                Ok(Some(message))
            }
            Ok(_) => Ok(None),
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}

impl<F: MessageFactory> Reliable<F::Message> for Connection<F> {}

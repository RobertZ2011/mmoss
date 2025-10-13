use anyhow::Result;
use async_trait::async_trait;
use std::marker::PhantomData;

pub mod tcp;
pub mod udp;

/// Message trait for serialization
pub trait Message: Send + Sync {
    fn serialize(&self, data: &mut [u8]) -> Result<usize>;
}

/// Message factory trait for deserialization
pub trait MessageFactory<Context = ()>: Send {
    type Message: Message;

    fn deserialize(&self, context: &Context, data: &[u8]) -> Result<(Self::Message, usize)>;
}

/// Unreliable transport for sending and receiving messages of type `M`
#[async_trait]
pub trait Unreliable<M: Message>: Send {
    async fn send(&mut self, message: &M) -> Result<()>;
    async fn receive(&mut self) -> Result<M>;
    fn try_receive(&mut self) -> Result<Option<M>>;
}

/// Reliable transport for sending and receiving messages of type `M`
pub trait Reliable<M: Message>: Unreliable<M> {}

pub struct Addressed<A: Send, M: Message> {
    pub address: A,
    pub message: M,
}

impl<A: Send + Sync, M: Message> Addressed<A, M> {
    pub fn new(address: A, message: M) -> Self {
        Self { address, message }
    }
}

impl<A: Send + Sync, M: Message> Message for Addressed<A, M> {
    fn serialize(&self, data: &mut [u8]) -> Result<usize> {
        self.message.serialize(data)
    }
}

pub struct AddressedFactory<A, F: MessageFactory> {
    pub factory: F,
    _marker: PhantomData<A>,
}

impl<A, F: MessageFactory> AddressedFactory<A, F> {
    pub fn new(factory: F) -> Self {
        Self {
            factory,
            _marker: PhantomData,
        }
    }
}

impl<A: Clone + Send + Sync, F: MessageFactory> MessageFactory<A> for AddressedFactory<A, F> {
    type Message = Addressed<A, F::Message>;

    fn deserialize(&self, context: &A, data: &[u8]) -> Result<(Self::Message, usize)> {
        let (message, len) = self.factory.deserialize(&(), data)?;
        Ok((
            Addressed {
                address: context.clone(),
                message,
            },
            len,
        ))
    }
}

impl Message for String {
    fn serialize(&self, data: &mut [u8]) -> Result<usize> {
        Ok(bincode::encode_into_slice(
            self,
            data,
            bincode::config::standard(),
        )?)
    }
}

/// String factory that creates a new string every time
pub struct StringFactoryNew;

impl MessageFactory for StringFactoryNew {
    type Message = String;

    fn deserialize(&self, _context: &(), data: &[u8]) -> Result<(Self::Message, usize)> {
        let (message, len): (Self::Message, _) =
            bincode::decode_from_slice(data, bincode::config::standard())?;
        Ok((message, len))
    }
}

impl Message for Vec<u8> {
    fn serialize(&self, data: &mut [u8]) -> Result<usize> {
        Ok(bincode::encode_into_slice(
            self,
            data,
            bincode::config::standard(),
        )?)
    }
}

pub struct VecU8FactoryNew;

impl MessageFactory for VecU8FactoryNew {
    type Message = Vec<u8>;

    fn deserialize(&self, _context: &(), data: &[u8]) -> Result<(Self::Message, usize)> {
        let (message, len): (Self::Message, _) =
            bincode::decode_from_slice(data, bincode::config::standard())?;
        Ok((message, len))
    }
}

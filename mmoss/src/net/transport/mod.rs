use anyhow::Result;
use async_trait::async_trait;
use std::{io, marker::PhantomData};

pub mod udp;

/// Message trait for serialization
pub trait Message {
    fn serialize(&self, writer: &mut impl io::Write) -> Result<()>;
}

/// Message factory trait for deserialization
pub trait MessageFactory<Context = ()> {
    type Message: Message;

    fn deserialize(&self, context: &Context, reader: &mut impl io::Read) -> Result<Self::Message>;
}

/// Unreliable transport for sending and receiving messages of type `M`
#[async_trait(?Send)]
pub trait Unreliable<M: Message> {
    async fn send(&mut self, message: M) -> Result<()>;
    async fn receive(&mut self) -> Result<M>;
    fn try_receive(&mut self) -> Result<Option<M>>;
}

pub struct Addressed<A, M: Message> {
    pub address: A,
    pub message: M,
}

impl<A, M: Message> Addressed<A, M> {
    pub fn new(address: A, message: M) -> Self {
        Self { address, message }
    }
}

impl<A, M: Message> Message for Addressed<A, M> {
    fn serialize(&self, writer: &mut impl std::io::Write) -> Result<()> {
        self.message.serialize(writer)
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

impl<A: Clone, F: MessageFactory> MessageFactory<A> for AddressedFactory<A, F> {
    type Message = Addressed<A, F::Message>;

    fn deserialize(&self, context: &A, reader: &mut impl std::io::Read) -> Result<Self::Message> {
        Ok(Addressed {
            address: context.clone(),
            message: self.factory.deserialize(&(), reader)?,
        })
    }
}

impl Message for String {
    fn serialize(&self, writer: &mut impl io::Write) -> Result<()> {
        let _ = bincode::encode_into_std_write(self, writer, bincode::config::standard())?;
        Ok(())
    }
}

/// String factory that creates a new string every time
pub struct StringFactoryNew;

impl MessageFactory for StringFactoryNew {
    type Message = String;

    fn deserialize(&self, _context: &(), reader: &mut impl io::Read) -> Result<Self::Message> {
        Ok(bincode::decode_from_std_read(
            reader,
            bincode::config::standard(),
        )?)
    }
}

impl Message for Vec<u8> {
    fn serialize(&self, writer: &mut impl io::Write) -> Result<()> {
        let _ = bincode::encode_into_std_write(self, writer, bincode::config::standard())?;
        Ok(())
    }
}

pub struct VecU8FactoryNew;

impl MessageFactory for VecU8FactoryNew {
    type Message = Vec<u8>;

    fn deserialize(&self, _context: &(), reader: &mut impl io::Read) -> Result<Self::Message> {
        Ok(bincode::decode_from_std_read(
            reader,
            bincode::config::standard(),
        )?)
    }
}

use anyhow::Result;
use bevy::ecs::{
    component::{Component, ComponentId},
    world::World,
};
use bevy_trait_query::queryable;
use bincode::{Decode, Encode};
use mmoss_proc_macros::Replicated;

use crate::net::transport::{Message as MessageTrait, MessageFactory as MessageFactoryTrait};

pub mod client;
pub mod convert;
pub mod server;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Decode, Encode)]
#[repr(transparent)]
pub struct Id(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Decode, Encode, Component)]
#[repr(transparent)]
pub struct MobType(pub u32);

#[queryable]
pub trait Replicated {
    fn id(&self) -> Id;
    fn component_id(&self, world: &World) -> ComponentId;

    fn serialize(&self, data: &mut [u8]) -> Result<usize>;
    fn replicate(&mut self, data: &[u8]) -> Result<usize>;
}

#[derive(Debug, Clone, Decode, Encode)]
pub struct UpdateData {
    pub id: Id,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Decode, Encode)]
pub struct SpawnData {
    pub mob_type: MobType,
    pub replicated: Vec<(usize, Id, Vec<u8>)>,
}

#[derive(Debug, Clone, Decode, Encode)]
pub enum Message {
    Spawn(SpawnData),
    Update(UpdateData),
}

impl MessageTrait for Message {
    fn serialize(&self, data: &mut [u8]) -> Result<usize> {
        Ok(bincode::encode_into_slice(
            self,
            data,
            bincode::config::standard(),
        )?)
    }
}

pub struct MessageFactoryNew;

impl MessageFactoryTrait for MessageFactoryNew {
    type Message = Message;

    fn deserialize(&self, _context: &(), _data: &[u8]) -> Result<(Self::Message, usize)> {
        Ok(bincode::decode_from_slice(
            _data,
            bincode::config::standard(),
        )?)
    }
}

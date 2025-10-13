use bevy::ecs::component::Component;
use bincode::{Decode, Encode};
use mmoss::{
    self,
    replication::{Id, Replicated},
};

use anyhow::Result;

#[derive(Debug, Clone, PartialEq, Encode, Decode, Default)]
pub struct ReplicatedData {
    pub rotation: f32,
    pub position: (i32, i32),
}

#[derive(Debug, Clone, Component)]
pub struct ReplicatedComponent {
    pub id: Id,
    pub replicated: ReplicatedData,
}

impl ReplicatedComponent {
    pub fn new(id: Id) -> Self {
        Self {
            id,
            replicated: ReplicatedData::default(),
        }
    }
}

impl Replicated for ReplicatedComponent {
    fn id(&self) -> mmoss::replication::Id {
        self.id
    }

    fn serialize(&self, writer: &mut [u8]) -> Result<usize> {
        Ok(bincode::encode_into_slice(
            &self.replicated,
            writer,
            bincode::config::standard(),
        )?)
    }

    fn replicate(&mut self, reader: &[u8]) -> Result<usize> {
        let (message, len) = bincode::decode_from_slice(reader, bincode::config::standard())?;
        self.replicated = message;
        Ok(len)
    }
}

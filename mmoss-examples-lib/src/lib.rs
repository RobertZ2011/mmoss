use mmoss::{
    self,
    replication::{Id, Replicated},
};

use anyhow::Result;

#[derive(Debug, Clone, PartialEq, bincode::Encode, bincode::Decode)]
pub struct ReplicatedData {
    pub rotation: f32,
    pub position: (i32, i32),
}

pub struct Square {
    pub id: Id,
    pub replicated: ReplicatedData,
}

impl Replicated for Square {
    fn id(&self) -> mmoss::replication::Id {
        self.id
    }

    fn serialize(&self, writer: &mut impl std::io::Write) -> Result<()> {
        bincode::encode_into_std_write(&self.replicated, writer, bincode::config::standard())?;
        Ok(())
    }

    fn replicate(&mut self, reader: &mut impl std::io::Read) -> Result<()> {
        bincode::decode_from_std_read(reader, bincode::config::standard())
            .map(|data: ReplicatedData| self.replicated = data)?;
        Ok(())
    }
}

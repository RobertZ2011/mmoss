use std::io;

use anyhow::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Id(pub u64);

pub trait Replicated {
    fn id(&self) -> Id;
    fn serialize(&self, writer: &mut impl io::Write) -> Result<()>;
    fn replicate(&mut self, reader: &mut impl io::Read) -> Result<()>;
}

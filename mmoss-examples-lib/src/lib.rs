use bevy::ecs::component::Component;
use bincode::{Decode, Encode};
use mmoss::{
    self,
    replication::{Id, Replicated},
};

use sdl2::render::Canvas;

use anyhow::{Result, anyhow};

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

#[derive(Debug, Clone, Component)]
pub struct RenderComponent;

impl RenderComponent {
    pub fn render(
        &self,
        canvas: &mut Canvas<sdl2::video::Window>,
        rotation: f32,
        position: (i32, i32),
    ) -> Result<()> {
        canvas
            .draw_line(
                position,
                (
                    (50.0 * rotation.to_radians().cos()) as i32 + position.0,
                    (50.0 * rotation.to_radians().sin()) as i32 + position.1,
                ),
            )
            .map_err(|e| anyhow!("Failed to draw line: {}", e))?;
        Ok(())
    }
}

pub mod mob {
    use bevy::ecs::world::World;
    use mmoss::replication::{Id, MobType, Replicated};

    use super::*;

    pub const SQUARE_TYPE: MobType = MobType(5);

    pub fn square_client(world: &mut World, replicated: &[(Id, Vec<u8>)]) -> anyhow::Result<()> {
        if replicated.len() != 1 {
            return Err(anyhow::anyhow!(
                "Expected 1 replicated component, got {}",
                replicated.len()
            ));
        }

        let mut replicated_component = ReplicatedComponent::new(replicated[0].0);
        replicated_component.replicate(&replicated[0].1)?;

        let mut entity = world.spawn((SQUARE_TYPE, replicated_component, RenderComponent));
        for (id, data) in replicated {
            entity.insert(super::ReplicatedComponent {
                id: *id,
                replicated: bincode::decode_from_slice(data, bincode::config::standard())?.0,
            });
            entity.insert(super::RenderComponent);
        }
        Ok(())
    }

    pub fn square_server(
        world: &mut World,
        replicated_data: (Id, ReplicatedData),
    ) -> anyhow::Result<()> {
        let mut replicated_component = ReplicatedComponent::new(replicated_data.0);
        replicated_component.replicated = replicated_data.1;

        world.spawn((SQUARE_TYPE, replicated_component, RenderComponent));
        Ok(())
    }
}

use bevy::ecs::component::Component;
use bincode::{Decode, Encode};
use mmoss::{self, replication::Id};

use mmoss::replication;
use mmoss_proc_macros::Replicated;
use sdl2::{pixels::Color, render::Canvas};

use anyhow::{Result, anyhow};

#[derive(Debug, Clone, PartialEq, Encode, Decode, Default)]
pub struct ReplicatedData {
    pub rotation: f32,
    pub position: (i32, i32),
}

#[derive(Debug, Clone, Component, Replicated)]
pub struct ReplicatedComponent {
    #[replication_id]
    pub id: Id,
    #[replicated]
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

#[derive(Debug, Clone, Component, Decode, Encode, Replicated)]
pub struct RenderComponent {
    #[replication_id]
    id: Id,
    #[replicated]
    pub color: (u8, u8, u8),
}

impl RenderComponent {
    pub fn new(id: Id) -> Self {
        Self {
            id,
            color: (0, 0, 0),
        }
    }
}

impl RenderComponent {
    pub fn render(
        &self,
        canvas: &mut Canvas<sdl2::video::Window>,
        rotation: f32,
        position: (i32, i32),
        (r, g, b): (u8, u8, u8),
    ) -> Result<()> {
        canvas.set_draw_color(Color::RGB(r, g, b));
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
    use bevy::ecs::{entity::Entity, world::World};
    use mmoss::replication::{Id, MobType, Replicated};

    use super::*;

    pub const SQUARE_TYPE: MobType = MobType(5);

    pub fn square_client(
        world: &mut World,
        replicated: &[(usize, Id, Vec<u8>)],
    ) -> anyhow::Result<Entity> {
        if replicated.len() != 2 {
            return Err(anyhow::anyhow!(
                "Expected 2 replicated components, got {}",
                replicated.len()
            ));
        }

        let replicated_index = replicated
            .iter()
            .position(|(index, _, _)| {
                *index == world.component_id::<ReplicatedComponent>().unwrap().index()
            })
            .ok_or_else(|| anyhow::anyhow!("ReplicatedComponent ID not found"))?;
        let mut replicated_component = ReplicatedComponent::new(replicated[replicated_index].1);
        replicated_component.replicate(&replicated[replicated_index].2)?;

        let render_index = replicated
            .iter()
            .position(|(index, _, _)| {
                *index == world.component_id::<RenderComponent>().unwrap().index()
            })
            .ok_or_else(|| anyhow::anyhow!("RenderComponent ID not found"))?;
        let mut render_component = RenderComponent::new(replicated[render_index].1);
        render_component.replicate(&replicated[render_index].2)?;

        Ok(world
            .spawn((SQUARE_TYPE, replicated_component, render_component))
            .id())
    }

    pub fn square_server(
        world: &mut World,
        replicated_data: (Id, ReplicatedData),
        render_data: (Id, (u8, u8, u8)),
    ) -> anyhow::Result<Entity> {
        let mut replicated_component = ReplicatedComponent::new(replicated_data.0);
        replicated_component.replicated = replicated_data.1;

        let mut render_component = RenderComponent::new(render_data.0);
        render_component.color = render_data.1;
        Ok(world
            .spawn((SQUARE_TYPE, replicated_component, render_component))
            .id())
    }
}

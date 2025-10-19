use bevy::ecs::component::Component;
use mmoss::{
    self,
    physics::TransformComponent,
    replication::{ComponentType, Id},
};

use mmoss::replication;
use mmoss_proc_macros::Replicated;
use sdl2::rect::Rect;
use sdl2::{pixels::Color, render::Canvas};

use anyhow::{Result, anyhow};

const RENDER_COMPONENT_TYPE: ComponentType = ComponentType(100);

#[derive(Debug, Clone, Component, Replicated)]
#[component_type(RENDER_COMPONENT_TYPE)]
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
        transform: &dyn TransformComponent,
    ) -> Result<()> {
        let translation = transform.transform().translation;
        canvas.set_draw_color(Color::RGB(self.color.0, self.color.1, self.color.2));
        canvas
            .draw_rect(Rect::from_center(
                (translation.x as i32, translation.y as i32),
                50,
                50,
            ))
            .map_err(|e| anyhow!("Failed to draw rect: {}", e))?;
        Ok(())
    }
}

pub mod mob {
    use async_trait::async_trait;
    use bevy::ecs::entity::Entity;
    use mmoss::{
        core::{self, component_type::physics::DYNAMIC_ACTOR_PROXY_COMPONENT_TYPE},
        physics::{
            self, DynamicActorComponent, Transform, World as _, proxy::DynamicActorComponentProxy,
        },
        replication::{Id, MobType, Replicated, client::FactoryEntry},
    };

    use super::*;

    pub const SQUARE_TYPE: MobType = MobType(5);

    pub struct SquareClient;

    #[async_trait(?Send)]
    impl<W: core::WorldContainer> FactoryEntry<W> for SquareClient {
        async fn construct(
            &self,
            world: &mut W,
            replicated: &[(ComponentType, Id, Vec<u8>)],
        ) -> anyhow::Result<Entity> {
            if replicated.len() != 2 {
                return Err(anyhow::anyhow!(
                    "Expected 2 replicated components, got {}",
                    replicated.len()
                ));
            }

            let replicated_index = replicated
                .iter()
                .position(|(index, _, _)| *index == DYNAMIC_ACTOR_PROXY_COMPONENT_TYPE)
                .ok_or_else(|| anyhow::anyhow!("ReplicatedComponent ID not found"))?;
            let mut dynamic_actor_component =
                DynamicActorComponentProxy::new(replicated[replicated_index].1);
            dynamic_actor_component.replicate(&replicated[replicated_index].2)?;

            let render_index = replicated
                .iter()
                .position(|(index, _, _)| *index == RENDER_COMPONENT_TYPE)
                .ok_or_else(|| anyhow::anyhow!("RenderComponent ID not found"))?;
            let mut render_component = RenderComponent::new(replicated[render_index].1);
            render_component.replicate(&replicated[render_index].2)?;

            Ok(world
                .world_mut()
                .spawn((SQUARE_TYPE, dynamic_actor_component, render_component))
                .id())
        }
    }

    pub async fn square_server(
        world: &mut (impl core::WorldContainer + physics::WorldContainer),
        transform_data: (Id, Transform),
        render_data: (Id, (u8, u8, u8)),
    ) -> anyhow::Result<Entity> {
        let entity = core::WorldContainer::world_mut(world).spawn_empty().id();

        let mut dynamic_actor_component = physics::WorldContainer::world_mut(world)
            .create_dynamic_actor_component(
                entity,
                transform_data.0,
                &transform_data.1,
                10.0,
                &physics::Material {
                    static_friction: 0.5,
                    dynamic_friction: 0.5,
                    restitution: 0.6,
                },
                &[(
                    physics::Shape::Sphere(physics::SphereShape { radius: 25.0 }),
                    Transform::default(),
                )],
            )
            .await?;
        *dynamic_actor_component.transform_mut() = transform_data.1;

        let mut render_component = RenderComponent::new(render_data.0);
        render_component.color = render_data.1;

        let mut entity = core::WorldContainer::world_mut(world)
            .get_entity_mut(entity)
            .unwrap();
        entity.insert((SQUARE_TYPE, dynamic_actor_component, render_component));
        Ok(entity.id())
    }

    pub fn square_server_no_physics(
        world: &mut impl core::WorldContainer,
        replicated_data: (Id, Transform),
        render_data: (Id, (u8, u8, u8)),
    ) -> anyhow::Result<Entity> {
        let mut dynamic_actor_component = DynamicActorComponentProxy::new(replicated_data.0);
        dynamic_actor_component.transform = replicated_data.1;

        let mut render_component = RenderComponent::new(render_data.0);
        render_component.color = render_data.1;
        Ok(core::WorldContainer::world_mut(world)
            .spawn((SQUARE_TYPE, dynamic_actor_component, render_component))
            .id())
    }
}

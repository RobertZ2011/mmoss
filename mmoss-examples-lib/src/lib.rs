use async_trait::async_trait;
use bevy::ecs::component::Component;
use mmoss::{
    self,
    physics::TransformComponent,
    replication::{ComponentType, Id, Replicated as _},
};

use mmoss::replication;
use mmoss_proc_macros::Replicated;
use sdl2::rect::Rect;
use sdl2::{pixels::Color, render::Canvas};

#[cfg(feature = "ffi")]
pub mod ffi;

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
                (translation.x as i32 * 10, translation.y as i32 * 10),
                50,
                50,
            ))
            .map_err(|e| anyhow!("Failed to draw rect: {}", e))?;
        Ok(())
    }
}

pub fn register_factory_components(
    factory: &mut mmoss::replication::client::factory::component::Factory<
        impl mmoss::core::WorldContainer,
    >,
) {
    factory.register_component(RENDER_COMPONENT_TYPE, RenderComponentFactory);
}

pub struct RenderComponentFactory;

#[async_trait(?Send)]
impl<W: mmoss::core::WorldContainer> mmoss::replication::client::factory::component::Entry<W>
    for RenderComponentFactory
{
    async fn add_component(
        &self,
        mut entity: bevy::ecs::world::EntityWorldMut<'_>,
        replication_id: Id,
        data: &Vec<u8>,
    ) -> Result<()> {
        let mut component = RenderComponent::new(replication_id);
        component.replicate(data)?;
        entity.insert(component);
        Ok(())
    }
}

pub mod mob {
    use async_trait::async_trait;
    use bevy::ecs::entity::Entity;
    use mmoss::{
        core,
        physics::{
            self, DynamicActorComponent, Transform, World as _, proxy::DynamicActorComponentProxy,
        },
        replication::{Id, MobType, client::factory::mob::Entry as MobFactoryEntry},
    };

    use super::*;

    pub const SQUARE_TYPE: MobType = MobType(5);

    pub struct SquareClient;

    #[async_trait(?Send)]
    impl<W: core::WorldContainer> MobFactoryEntry<W> for SquareClient {
        async fn construct(&self, world: &mut W) -> anyhow::Result<Entity> {
            Ok(world.world_mut().spawn(SQUARE_TYPE).id())
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
                    physics::Shape::Sphere(physics::SphereShape { radius: 2.5 }),
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

//! Proxy types for physics components
//!
//! Clients may not need to have full physics simulation capabilities, but they
//! still need to be able to represent physics actors. This module provides
//! proxy types that only replicated a remote component.

use crate::{
    core::{
        WorldContainer,
        component_type::physics::{
            DYNAMIC_ACTOR_PROXY_COMPONENT_TYPE, STATIC_ACTOR_PROXY_COMPONENT_TYPE,
        },
    },
    physics::{DynamicActorComponent, StaticActorComponent, TransformComponent},
    replication::Replicated,
};
use anyhow::Result;
use async_trait::async_trait;
use bevy::ecs::{component::Component, world::EntityWorldMut};
use mmoss_proc_macros::Replicated;

use crate::{physics::Transform, replication, replication::Id};

use replication::client::factory::component::Entry as ComponentFactory;

#[derive(Debug, Clone, Component, Replicated)]
#[component_type(DYNAMIC_ACTOR_PROXY_COMPONENT_TYPE)]
pub struct DynamicActorComponentProxy {
    #[replication_id]
    pub id: Id,
    #[replicated]
    pub transform: Transform,
}

impl DynamicActorComponentProxy {
    pub fn new(id: Id) -> Self {
        Self {
            id,
            transform: Transform::default(),
        }
    }
}

impl TransformComponent for DynamicActorComponentProxy {
    fn transform(&self) -> &Transform {
        &self.transform
    }
}

impl DynamicActorComponent for DynamicActorComponentProxy {
    fn transform_mut(&mut self) -> &mut Transform {
        &mut self.transform
    }
}

pub struct DynamicActorComponentProxyFactory;

#[async_trait(?Send)]
impl<W: WorldContainer> ComponentFactory<W> for DynamicActorComponentProxyFactory {
    async fn add_component(
        &self,
        mut entity: EntityWorldMut<'_>,
        replication_id: Id,
        data: &Vec<u8>,
    ) -> Result<()> {
        let mut component = DynamicActorComponentProxy::new(replication_id);

        component.replicate(data)?;
        entity.insert(component);
        Ok(())
    }
}

#[derive(Debug, Clone, Component, Replicated)]
#[component_type(STATIC_ACTOR_PROXY_COMPONENT_TYPE)]
pub struct StaticActorComponentProxy {
    #[replication_id]
    pub id: Id,
    #[replicated]
    pub transform: Transform,
}

impl StaticActorComponentProxy {
    pub fn new(id: Id) -> Self {
        Self {
            id,
            transform: Transform::default(),
        }
    }
}

impl TransformComponent for StaticActorComponentProxy {
    fn transform(&self) -> &Transform {
        &self.transform
    }
}

impl StaticActorComponent for StaticActorComponentProxy {}

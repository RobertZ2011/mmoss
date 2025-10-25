use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;
use bevy::ecs::{entity::Entity, world::EntityWorldMut};

use crate::{
    core::{WorldContainer, component_type::physics::DYNAMIC_ACTOR_PROXY_COMPONENT_TYPE},
    physics::proxy::DynamicActorComponentProxyFactory,
    replication::{ComponentType, Id},
};

#[async_trait(?Send)]
pub trait Entry<W: WorldContainer> {
    async fn add_component(
        &self,
        entity: EntityWorldMut<'_>,
        replication_id: Id,
        data: &Vec<u8>,
    ) -> Result<()>;
}

pub struct Factory<W: WorldContainer> {
    prototypes: HashMap<ComponentType, Box<dyn Entry<W>>>,
}

impl<W: WorldContainer> Factory<W> {
    pub fn new() -> Self {
        Self {
            prototypes: HashMap::new(),
        }
    }

    pub fn register_component(
        &mut self,
        component_type: ComponentType,
        constructor: impl Entry<W> + 'static,
    ) {
        self.prototypes
            .insert(component_type, Box::new(constructor));
    }

    pub async fn add_component(
        &self,
        world: &mut W,
        entity: Entity,
        component_type: ComponentType,
        replication_id: Id,
        data: &Vec<u8>,
    ) -> Result<()> {
        let constructor = self.prototypes.get(&component_type).ok_or_else(|| {
            anyhow::anyhow!(
                "No prototype registered for component type {:?}",
                component_type
            )
        })?;

        let entity = world.world_mut().entity_mut(entity);
        constructor
            .add_component(entity, replication_id, data)
            .await
    }
}

pub fn register_default_factory_components<W: WorldContainer>(factory: &mut Factory<W>) {
    factory.register_component(
        DYNAMIC_ACTOR_PROXY_COMPONENT_TYPE,
        DynamicActorComponentProxyFactory,
    );
}

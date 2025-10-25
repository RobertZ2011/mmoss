use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;
use bevy::ecs::entity::Entity;

use crate::{core::WorldContainer, replication::MobType};

#[async_trait(?Send)]
pub trait Entry<W: WorldContainer> {
    async fn construct(&self, world: &mut W) -> Result<Entity>;
}

pub struct Factory<W: WorldContainer> {
    prototypes: HashMap<MobType, Box<dyn Entry<W>>>,
}

impl<W: WorldContainer> Factory<W> {
    pub fn new() -> Self {
        Self {
            prototypes: HashMap::new(),
        }
    }

    pub fn register_mob(&mut self, mob_type: MobType, constructor: impl Entry<W> + 'static) {
        self.prototypes.insert(mob_type, Box::new(constructor));
    }

    pub async fn construct(&self, world: &mut W, mob_type: MobType) -> Result<Entity> {
        let constructor = self.prototypes.get(&mob_type).ok_or_else(|| {
            anyhow::anyhow!("No prototype registered for mob type {:?}", mob_type)
        })?;
        constructor.construct(world).await
    }
}

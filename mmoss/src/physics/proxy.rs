//! Proxy types for physics components
//!
//! Clients may not need to have full physics simulation capabilities, but they
//! still need to be able to represent physics actors. This module provides
//! proxy types that only replicated a remote component.

use crate::{
    core::component_type::physics::{
        DYNAMIC_ACTOR_PROXY_COMPONENT_TYPE, STATIC_ACTOR_PROXY_COMPONENT_TYPE,
    },
    physics::{DynamicActorComponent, StaticActorComponent, TransformComponent},
};
use bevy::ecs::component::Component;
use mmoss_proc_macros::Replicated;

use crate::{physics::Transform, replication, replication::Id};

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

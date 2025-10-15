use bevy::ecs::component::Component;
use mmoss_proc_macros::Replicated;

use crate::replication::{self, Id, MobType};

extern crate self as mmoss;

#[derive(Debug, Clone, Component, Replicated)]
pub struct MobComponent {
    #[replication_id]
    pub id: Id,
    // Not #[replicated] because the server handles this field directly
    pub mob_type: MobType,
    #[replicated(into_from = crate::replication::convert::Vec3)]
    pub position: bevy::math::Vec3,
    #[replicated(into_from = crate::replication::convert::Quat)]
    pub rotation: bevy::math::Quat,
}

impl MobComponent {
    pub fn new(id: Id, mob_type: MobType) -> Self {
        Self {
            id,
            mob_type,
            position: bevy::math::Vec3::ZERO,
            rotation: bevy::math::Quat::IDENTITY,
        }
    }
}

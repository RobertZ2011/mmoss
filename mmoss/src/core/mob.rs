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
}

impl MobComponent {
    pub fn new(id: Id, mob_type: MobType) -> Self {
        Self { id, mob_type }
    }
}

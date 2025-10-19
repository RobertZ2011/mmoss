use crate::replication::ComponentType;

/// Replication-related component types, range 0x0000 - 0x00FF
pub mod replication {}

pub mod physics {
    use super::*;

    pub const DYNAMIC_ACTOR_COMPONENT_TYPE: ComponentType = ComponentType(6);
    pub const STATIC_ACTOR_COMPONENT_TYPE: ComponentType = ComponentType(5);
    pub const DYNAMIC_ACTOR_PROXY_COMPONENT_TYPE: ComponentType = ComponentType(7);
    pub const STATIC_ACTOR_PROXY_COMPONENT_TYPE: ComponentType = ComponentType(8);
}

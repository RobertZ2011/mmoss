pub mod component_type;

pub trait WorldContainer {
    fn world(&self) -> &bevy::ecs::world::World;
    fn world_mut(&mut self) -> &mut bevy::ecs::world::World;
}

impl WorldContainer for bevy::ecs::world::World {
    fn world(&self) -> &bevy::ecs::world::World {
        self
    }

    fn world_mut(&mut self) -> &mut bevy::ecs::world::World {
        self
    }
}

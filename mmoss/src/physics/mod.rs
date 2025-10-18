use bevy::{
    ecs::component::Component,
    math::{Quat, Vec3},
};

pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
}

pub struct Material {
    pub static_friction: f32,
    pub dynamic_friction: f32,
    pub restitution: f32,
}

pub enum DynamicShape {}

pub enum StaticShape {}

/// Core actor trait for components
pub trait StaticActorComponent: Component {
    fn transform(&self) -> &Transform;
}

pub trait DynamicActorComponent: Component {
    fn transform(&self) -> &Transform;
    fn transform_mut(&mut self) -> &mut Transform;
}

pub trait Engine {
    type WorldType: World;

    fn create_world(&mut self, gravity: Vec3) -> Self::WorldType;
}

pub trait World {
    type StaticActorComponentType: StaticActorComponent;
    type DynamicActorComponentType: DynamicActorComponent;

    fn step(&mut self, delta_time: f32);
    fn update_world(&mut self, world: &mut bevy::ecs::world::World);

    fn create_dynamic_actor_component(
        &mut self,
        transform: Transform,
        material: Material,
        density: f32,
    ) -> Self::DynamicActorComponentType;

    fn create_static_actor_component(
        &mut self,
        transform: Transform,
        material: Material,
    ) -> Self::StaticActorComponentType;
}

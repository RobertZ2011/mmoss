use bevy::math::{Quat, Vec3};

pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
}

pub struct Material {
    pub static_friction: f32,
    pub dynamic_friction: f32,
    pub restitution: f32,
}

/// Core actor trait for components
pub trait Actor {
    fn transform(&self) -> &Transform;
}

pub trait DynamicActor: Actor {
    fn transform_mut(&mut self) -> &mut Transform;
}

pub trait Engine {
    fn create_world(&mut self) -> Box<dyn World>;
}

pub trait World {
    fn step(&mut self, delta_time: f32);

    fn create_dynamic_cube(
        &mut self,
        transform: Transform,
        material: Material,
        density: f32,
    ) -> Box<dyn DynamicActor>;

    fn create_static_plane(&mut self, transform: Transform, material: Material) -> Box<dyn Actor>;
}

use anyhow::Result;
use bevy::{
    ecs::entity::Entity,
    math::{Quat, Vec3},
};
use bevy_trait_query::queryable;
use bincode::{Decode, Encode};

pub mod proxy;

use crate::replication::{Id, convert};

#[derive(Debug, Clone)]
pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quat,
}

impl Encode for Transform {
    fn encode<E: bincode::enc::Encoder>(
        &self,
        encoder: &mut E,
    ) -> core::result::Result<(), bincode::error::EncodeError> {
        convert::Vec3::from(self.translation).encode(encoder)?;
        convert::Quat::from(self.rotation).encode(encoder)?;
        Ok(())
    }
}

impl<Context> Decode<Context> for Transform {
    fn decode<D: bincode::de::Decoder<Context = Context>>(
        decoder: &mut D,
    ) -> core::result::Result<Self, bincode::error::DecodeError> {
        let position = convert::Vec3::decode(decoder)?.into();
        let rotation = convert::Quat::decode(decoder)?.into();
        Ok(Transform {
            translation: position,
            rotation,
        })
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
        }
    }
}

pub struct Material {
    pub static_friction: f32,
    pub dynamic_friction: f32,
    pub restitution: f32,
}

pub struct BoxShape {
    pub half_extents: Vec3,
}

pub struct SphereShape {
    pub radius: f32,
}

pub struct CapsuleShape {
    pub half_height: f32,
    pub radius: f32,
}

pub struct PlaneShape {
    pub normal: Vec3,
    pub offset: f32,
}

pub enum Shape {
    Box(BoxShape),
    Sphere(SphereShape),
    Capsule(CapsuleShape),
}

#[queryable]
pub trait TransformComponent {
    fn transform(&self) -> &Transform;
}

/// Core actor trait for components
#[queryable]
pub trait StaticActorComponent: TransformComponent {}

#[queryable]
pub trait DynamicActorComponent: TransformComponent {
    fn transform_mut(&mut self) -> &mut Transform;
}

pub trait Engine {
    type WorldType: World;

    fn create_world(&mut self, gravity: Vec3) -> impl Future<Output = Result<Self::WorldType>>;
}

pub trait World {
    type StaticActorComponentType: StaticActorComponent;
    type DynamicActorComponentType: DynamicActorComponent;

    fn update_world(&mut self, world: &mut bevy::ecs::world::World, delta_time: f32) -> Result<()>;

    fn create_plane(
        &mut self,
        entity: Entity,
        replication_id: Id,
        material: &Material,
        plane: &PlaneShape,
    ) -> impl Future<Output = Result<Self::StaticActorComponentType>>;

    fn create_dynamic_actor_component(
        &mut self,
        entity: Entity,
        replication_id: Id,
        transform: &Transform,
        density: f32,
        material: &Material,
        shapes: &[(Shape, Transform)],
    ) -> impl Future<Output = Result<Self::DynamicActorComponentType>>;

    fn create_static_actor_component(
        &mut self,
        entity: Entity,
        replication_id: Id,
        transform: &Transform,
        material: &Material,
        shapes: &[(Shape, Transform)],
    ) -> impl Future<Output = Result<Self::StaticActorComponentType>>;
}

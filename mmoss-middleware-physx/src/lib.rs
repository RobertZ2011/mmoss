use std::{ptr::null, sync::Arc};

use anyhow::{Result, anyhow};
use bevy::{
    ecs::{component::Component, entity::Entity},
    math::Vec3,
};
use bevy_trait_query::One;
use log::{error, trace};
use mmoss::{
    core::component_type::physics::{DYNAMIC_ACTOR_COMPONENT_TYPE, STATIC_ACTOR_COMPONENT_TYPE},
    physics::{self, Shape, Transform},
    replication::{self, Id},
};
use mmoss_proc_macros::Replicated;
use physx::{prelude::*, traits::Class as _};
use physx_sys::PxSceneFlags;
use tokio::sync::Mutex;

type PxMaterial = physx::material::PxMaterial<Entity>;
type PxShape = physx::shape::PxShape<Entity, PxMaterial>;
type PxArticulationLink = physx::articulation_link::PxArticulationLink<Entity, PxShape>;
type PxRigidStatic = physx::rigid_static::PxRigidStatic<Entity, PxShape>;
type PxRigidDynamic = physx::rigid_dynamic::PxRigidDynamic<Entity, PxShape>;
type PxArticulationReducedCoordinate =
    physx::articulation_reduced_coordinate::PxArticulationReducedCoordinate<
        Entity,
        PxArticulationLink,
    >;
type PxScene = physx::scene::PxScene<
    *const std::ffi::c_void,
    PxArticulationLink,
    PxRigidStatic,
    PxRigidDynamic,
    PxArticulationReducedCoordinate,
    OnCollision,
    OnTrigger,
    OnConstraintBreak,
    OnWakeSleep,
    OnAdvance,
>;

/// Next up, the simulation event callbacks need to be defined, and possibly an
/// allocator callback as well.
struct OnCollision;
impl CollisionCallback for OnCollision {
    fn on_collision(
        &mut self,
        _header: &physx_sys::PxContactPairHeader,
        _pairs: &[physx_sys::PxContactPair],
    ) {
    }
}
struct OnTrigger;
impl TriggerCallback for OnTrigger {
    fn on_trigger(&mut self, _pairs: &[physx_sys::PxTriggerPair]) {}
}

struct OnConstraintBreak;
impl ConstraintBreakCallback for OnConstraintBreak {
    fn on_constraint_break(&mut self, _constraints: &[physx_sys::PxConstraintInfo]) {}
}
struct OnWakeSleep;
impl WakeSleepCallback<PxArticulationLink, PxRigidStatic, PxRigidDynamic> for OnWakeSleep {
    fn on_wake_sleep(
        &mut self,
        _actors: &[&physx::actor::ActorMap<PxArticulationLink, PxRigidStatic, PxRigidDynamic>],
        _is_waking: bool,
    ) {
    }
}

struct OnAdvance;
impl AdvanceCallback<PxArticulationLink, PxRigidDynamic> for OnAdvance {
    fn on_advance(
        &self,
        _actors: &[&physx::rigid_body::RigidBodyMap<PxArticulationLink, PxRigidDynamic>],
        _transforms: &[PxTransform],
    ) {
    }
}

pub struct InnerEngine<Allocator: AllocatorCallback> {
    pub physics: PhysicsFoundation<Allocator, PxShape>,
}

impl<Allocator: AllocatorCallback> InnerEngine<Allocator> {
    pub fn new(allocator: Allocator) -> Self {
        Self {
            physics: PhysicsFoundation::new(allocator),
        }
    }

    fn create_box_shape(
        &mut self,
        entity: Entity,
        shape: &physics::BoxShape,
        material: &physics::Material,
    ) -> Result<Owner<PxShape>> {
        let geometry = PxBoxGeometry::new(
            shape.half_extents.x,
            shape.half_extents.y,
            shape.half_extents.z,
        );
        let mut material = self.create_px_material(entity, material).unwrap();
        self.physics
            .create_shape(
                &geometry,
                &mut [&mut material],
                true,
                ShapeFlags::SimulationShape,
                entity,
            )
            .ok_or(anyhow!("Failed to create box shape"))
    }

    fn create_sphere_shape(
        &mut self,
        entity: Entity,
        shape: &physics::SphereShape,
        material: &physics::Material,
    ) -> Result<Owner<PxShape>> {
        let geometry = PxSphereGeometry::new(shape.radius);
        let mut material = self.create_px_material(entity, material).unwrap();
        self.physics
            .create_shape(
                &geometry,
                &mut [&mut material],
                true,
                ShapeFlags::SimulationShape,
                entity,
            )
            .ok_or(anyhow!("Failed to create sphere shape"))
    }

    fn create_capsule_shape(
        &mut self,
        entity: Entity,
        shape: &physics::CapsuleShape,
        material: &physics::Material,
    ) -> Result<Owner<PxShape>> {
        let geometry = PxCapsuleGeometry::new(shape.half_height, shape.radius);
        let mut material = self.create_px_material(entity, material).unwrap();
        self.physics
            .create_shape(
                &geometry,
                &mut [&mut material],
                true,
                ShapeFlags::SimulationShape,
                entity,
            )
            .ok_or(anyhow!("Failed to create capsule shape"))
    }

    fn create_px_material(
        &mut self,
        entity: Entity,
        material: &physics::Material,
    ) -> Result<Owner<PxMaterial>> {
        Ok(self
            .physics
            .create_material(
                material.static_friction,
                material.dynamic_friction,
                material.restitution,
                entity,
            )
            .ok_or(anyhow!("Failed to create material"))?)
    }

    fn attach_shapes(
        &mut self,
        entity: Entity,
        actor: &mut Owner<impl RigidActor<Shape = PxShape>>,
        shapes: &[(Shape, Transform)],
        material: &physics::Material,
    ) -> Result<()> {
        // TODO: support transforms for shapes
        for (shape, _transform) in shapes {
            match shape {
                Shape::Box(box_shape) => {
                    let mut shape = self.create_box_shape(entity, box_shape, material)?;
                    actor.attach_shape(&mut shape)
                }
                Shape::Sphere(sphere_shape) => {
                    let mut shape = self.create_sphere_shape(entity, sphere_shape, material)?;
                    actor.attach_shape(&mut shape)
                }
                Shape::Capsule(capsule_shape) => {
                    let mut shape = self.create_capsule_shape(entity, capsule_shape, material)?;
                    actor.attach_shape(&mut shape)
                }
            };
        }

        Ok(())
    }
}

pub struct Engine<Allocator: AllocatorCallback> {
    inner: Arc<Mutex<InnerEngine<Allocator>>>,
}

impl<Allocator: AllocatorCallback> Engine<Allocator> {
    pub fn new(allocator: Allocator) -> Self {
        Self {
            inner: Arc::new(Mutex::new(InnerEngine::new(allocator))),
        }
    }
}

impl<Allocator: AllocatorCallback> physics::Engine for Engine<Allocator> {
    type WorldType = World<Allocator>;

    async fn create_world(&mut self, gravity: Vec3) -> Result<Self::WorldType> {
        let mut engine = self.inner.lock().await;
        Ok(World {
            scene: engine
                .physics
                .create(SceneDescriptor {
                    gravity: PxVec3::new(gravity.x, gravity.y, gravity.z),
                    on_advance: Some(OnAdvance),
                    flags: PxSceneFlags::EnableActiveActors,
                    ..SceneDescriptor::new(std::ptr::null())
                })
                .ok_or(anyhow!("Failed to create world"))?,
            engine: self.inner.clone(),
        })
    }
}

pub struct World<Allocator: AllocatorCallback> {
    scene: Owner<PxScene>,
    engine: Arc<Mutex<InnerEngine<Allocator>>>,
}

impl<Allocator: AllocatorCallback> physics::World for World<Allocator> {
    type StaticActorComponentType = StaticActorComponent;
    type DynamicActorComponentType = DynamicActorComponent;

    fn update_world(&mut self, world: &mut bevy::ecs::world::World, delta_time: f32) -> Result<()> {
        let mut scratch = unsafe { ScratchBuffer::new(4) };
        self.scene
            .step(
                delta_time,
                None::<&mut physx_sys::PxBaseTask>,
                Some(&mut scratch),
                true,
            )
            .map_err(|e| anyhow!("Failed to step physics scene: {}", e))?;

        let mut query = world.query::<One<&mut dyn physics::DynamicActorComponent>>();
        let actors = self.scene.get_active_actors();
        trace!("Active actors: {}", actors.len());
        for actor in actors {
            if let Some(dynamic) = actor.as_rigid_dynamic() {
                let entity = dynamic.get_user_data();
                if let Ok(mut component) = query.get_mut(world, *entity) {
                    let position = dynamic.get_global_pose().translation();
                    let rotation = dynamic.get_global_pose().rotation();
                    component.transform_mut().translation =
                        Vec3::new(position.x(), position.y(), position.z());
                    component.transform_mut().rotation = bevy::math::Quat::from_xyzw(
                        rotation.x(),
                        rotation.y(),
                        rotation.z(),
                        rotation.w(),
                    );
                } else {
                    error!(
                        "Failed to find dynamic actor component for entity {:?}",
                        entity
                    );
                }
            }
        }

        Ok(())
    }

    async fn create_plane(
        &mut self,
        entity: Entity,
        replication_id: Id,
        material: &physics::Material,
        shape: &physics::PlaneShape,
    ) -> Result<StaticActorComponent> {
        let mut engine = self.engine.lock().await;
        let mut material = engine.create_px_material(entity, material)?;
        let actor = engine
            .physics
            .create_plane(
                PxVec3::new(shape.normal.x, shape.normal.y, shape.normal.z),
                shape.offset,
                &mut material,
                entity,
            )
            .ok_or(anyhow!("Failed to create plane"))?;
        self.scene.add_static_actor(actor);

        Ok(StaticActorComponent {
            id: replication_id,
            transform: Transform {
                translation: shape.normal * shape.offset,
                rotation: bevy::math::Quat::IDENTITY,
            },
        })
    }

    async fn create_dynamic_actor_component(
        &mut self,
        entity: Entity,
        replication_id: Id,
        transform: &Transform,
        density: f32,
        material: &physics::Material,
        shapes: &[(Shape, Transform)],
    ) -> Result<Self::DynamicActorComponentType> {
        if shapes.len() == 0 {
            return Err(anyhow!(
                "At least one shape is required to create a dynamic actor"
            ));
        }

        let mut engine = self.engine.lock().await;

        let mut actor = engine
            .physics
            .create_dynamic(
                &PxTransform::from_translation_rotation(
                    &PxVec3::new(
                        transform.translation.x,
                        transform.translation.y,
                        transform.translation.z,
                    ),
                    &PxQuat::new(
                        transform.rotation.x,
                        transform.rotation.y,
                        transform.rotation.z,
                        transform.rotation.w,
                    ),
                ),
                entity,
            )
            .ok_or(anyhow!("Failed to create dynamic actor"))?;

        engine.attach_shapes(entity, &mut actor, shapes, material)?;

        let updated = unsafe {
            physx_sys::PxRigidBodyExt_updateMassAndInertia_1(
                actor.as_mut_ptr(),
                density,
                null(),
                false,
            )
        };

        if !updated {
            return Err(anyhow!("Failed to update mass and inertia"));
        }

        self.scene.add_dynamic_actor(actor);

        Ok(DynamicActorComponent {
            id: replication_id,
            transform: transform.clone(),
        })
    }

    async fn create_static_actor_component(
        &mut self,
        entity: Entity,
        replication_id: Id,
        transform: &Transform,
        material: &physics::Material,
        shapes: &[(Shape, Transform)],
    ) -> Result<Self::StaticActorComponentType> {
        if shapes.len() == 0 {
            return Err(anyhow!(
                "At least one shape is required to create a static actor"
            ));
        }

        let mut engine = self.engine.lock().await;
        let mut actor = engine
            .physics
            .create_static(
                PxTransform::from_translation_rotation(
                    &PxVec3::new(
                        transform.translation.x,
                        transform.translation.y,
                        transform.translation.z,
                    ),
                    &PxQuat::new(
                        transform.rotation.x,
                        transform.rotation.y,
                        transform.rotation.z,
                        transform.rotation.w,
                    ),
                ),
                entity,
            )
            .ok_or(anyhow!("Failed to create static actor"))?;

        engine.attach_shapes(entity, &mut actor, shapes, material)?;
        self.scene.add_static_actor(actor);

        Ok(StaticActorComponent {
            id: replication_id,
            transform: transform.clone(),
        })
    }
}

#[derive(Component, Replicated)]
#[component_type(STATIC_ACTOR_COMPONENT_TYPE)]
pub struct StaticActorComponent {
    #[replication_id]
    pub id: Id,
    #[replicated]
    pub transform: Transform,
}

impl physics::TransformComponent for StaticActorComponent {
    fn transform(&self) -> &Transform {
        &self.transform
    }
}

impl physics::StaticActorComponent for StaticActorComponent {}

#[derive(Component, Replicated)]
#[component_type(DYNAMIC_ACTOR_COMPONENT_TYPE)]
pub struct DynamicActorComponent {
    #[replication_id]
    pub id: Id,
    #[replicated]
    pub transform: Transform,
}

impl physics::TransformComponent for DynamicActorComponent {
    fn transform(&self) -> &Transform {
        &self.transform
    }
}

impl physics::DynamicActorComponent for DynamicActorComponent {
    fn transform_mut(&mut self) -> &mut Transform {
        &mut self.transform
    }
}

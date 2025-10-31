use std::{
    ffi::{CStr, c_char},
    sync::Arc,
};

use bevy::ecs::entity::Entity;
use bevy_trait_query::RegisterExt as _;
use log::error;
use mmoss::{
    core::{self, WorldContainer}, net,
    physics::{TransformComponent, proxy::DynamicActorComponentProxy},
    replication::{self, MessageFactoryNew, Replicated, SpawnId, client::UpdateCallbacks},
};

use crate::types::{Quat, Vec3};

/// Combined bevy world and replication manager
///
/// The combination helps reduce the number of FFI types needed.
pub struct WorldObj {
    pub bevy_world: bevy::ecs::world::World,
    pub replication_manager: replication::client::Manager<bevy::ecs::world::World>,
    pub rtt: tokio::runtime::Runtime,
}

impl core::WorldContainer for WorldObj {
    fn world(&self) -> &bevy::ecs::world::World {
        &self.bevy_world
    }

    fn world_mut(&mut self) -> &mut bevy::ecs::world::World {
        &mut self.bevy_world
    }
}

pub struct MobFactoryBuilderObj {
    pub mob_factory: Box<replication::client::factory::mob::Factory<bevy::ecs::world::World>>,
}

#[repr(C)]
pub struct MobFactoryBuilderPtr;

#[unsafe(no_mangle)]
pub extern "C" fn mmoss_client_factory_builder_new() -> *mut MobFactoryBuilderPtr {
    let factory = MobFactoryBuilderObj {
        mob_factory: Box::new(replication::client::factory::mob::Factory::new()),
    };

    Box::into_raw(Box::new(factory)) as *mut MobFactoryBuilderPtr
}

#[unsafe(no_mangle)]
pub extern "C" fn mmoss_client_factory_builder_build(
    builder: *mut MobFactoryBuilderPtr,
) -> *mut MobFactoryPtr {
    if builder.is_null() {
        error!("Null builder passed to client_factory_builder_build");
        return std::ptr::null_mut();
    }

    let builder = unsafe { Box::from_raw(builder as *mut MobFactoryBuilderObj) };
    let factory = MobFactoryObj {
        factory: Arc::new(*builder.mob_factory),
    };

    Box::into_raw(Box::new(factory)) as *mut MobFactoryPtr
}

pub struct MobFactoryObj {
    pub factory: Arc<replication::client::factory::mob::Factory<bevy::ecs::world::World>>,
}

#[repr(C)]
pub struct MobFactoryPtr;


pub struct ComponentFactoryBuilderObj {
    pub component_factory: Box<replication::client::factory::component::Factory<bevy::ecs::world::World>>,
}

#[repr(C)]
pub struct ComponentFactoryBuilderPtr;

#[unsafe(no_mangle)]
pub extern "C" fn mmoss_client_component_factory_builder_new() -> *mut ComponentFactoryBuilderPtr {
    let mut factory = replication::client::factory::component::Factory::new();
    replication::client::factory::component::register_default_factory_components(&mut factory);
    
    let factory = ComponentFactoryBuilderObj {
        component_factory: Box::new(factory),
    };

    Box::into_raw(Box::new(factory)) as *mut ComponentFactoryBuilderPtr
}

#[unsafe(no_mangle)]
pub extern "C" fn mmoss_client_component_factory_builder_build(
    builder: *mut ComponentFactoryBuilderPtr,
) -> *mut ComponentFactoryPtr {
    if builder.is_null() {
        error!("Null builder passed to client_component_factory_builder_build");
        return std::ptr::null_mut();
    }

    let builder = unsafe { Box::from_raw(builder as *mut ComponentFactoryBuilderObj) };
    let factory = ComponentFactoryObj {
        factory: Arc::new(*builder.component_factory),
    };

    Box::into_raw(Box::new(factory)) as *mut ComponentFactoryPtr
}

pub struct ComponentFactoryObj {
    pub factory: Arc<replication::client::factory::component::Factory<bevy::ecs::world::World>>,
}

#[repr(C)]
pub struct ComponentFactoryPtr;

#[repr(C)]
pub struct WorldPtr;

#[repr(C)]
pub struct BevyWorldPtr;

#[unsafe(no_mangle)]
pub extern "C" fn mmoss_client_world_new(
    mob_factory: *const MobFactoryPtr,
    component_factory: *const ComponentFactoryPtr,
    addr: *const c_char,
) -> *mut WorldPtr {
    if mob_factory.is_null() {
        error!("Null factory passed to client_world_new");
        return std::ptr::null_mut();
    }

    if addr.is_null() {
        error!("Null addr passed to client_world_new");
        return std::ptr::null_mut();
    }

    let rtt = tokio::runtime::Runtime::new().unwrap();

    let addr_str = unsafe { CStr::from_ptr(addr) }.to_str();
    if let Err(e) = addr_str {
        error!("Invalid address string: {:?}", e);
        return std::ptr::null_mut();
    }

    let addr_str = addr_str.unwrap();
    let transport = rtt.block_on(async {
        net::transport::tcp::Connection::connect(addr_str, MessageFactoryNew).await
    });

    if let Err(e) = transport {
        log::error!("Error connecting to server: {:?}", e);
        return std::ptr::null_mut();
    }

    let mut bevy_world = bevy::ecs::world::World::new();
    bevy_world.register_component_as::<dyn TransformComponent, DynamicActorComponentProxy>();
    bevy_world.register_component_as::<dyn Replicated, DynamicActorComponentProxy>();

    let mob_factory = unsafe { &*(mob_factory as *const MobFactoryObj) };
    let component_factory = unsafe { &*(component_factory as *const ComponentFactoryObj) };
    let (replication_manager, incoming) = replication::client::Manager::new(
        Box::new(transport.unwrap()),
        mob_factory.factory.clone(),
        component_factory.factory.clone(),
    );

    // Start processing incoming messages
    rtt.spawn(async move {
        let mut incoming = incoming;
        loop {
            if let Err(e) = incoming.process_incoming().await {
                error!("Error processing incoming message: {:?}", e);
            }
        }
    });

    let world = WorldObj {
        bevy_world,
        replication_manager,
        rtt,
    };

    Box::into_raw(Box::new(world)) as *mut WorldPtr
}

#[unsafe(no_mangle)]
pub extern "C" fn mmoss_client_world_destroy(world: *mut WorldPtr) {
    if world.is_null() {
        return;
    }

    unsafe {
        let _dropped = Box::from_raw(world as *mut WorldObj);
    }
}

pub struct ClientWorldUpdateCallbacks {
    pub on_spawn: Option<unsafe extern "C" fn(entity: u64, mob_type: u32)>,
    pub on_component_updated: Option<unsafe extern "C" fn(entity: u64, id: u32)>,
    pub on_component_added:
        Option<unsafe extern "C" fn(entity: u64, spawn_id: u32, component_type: u32, id: u32)>,
}

impl UpdateCallbacks for ClientWorldUpdateCallbacks {
    fn on_component_updated(&mut self, entity: Entity, _spawn_id: SpawnId, id: replication::Id) {
        if let Some(callback) = self.on_component_updated {
            unsafe {
                callback(entity.to_bits(), id.0);
            }
        }
    }

    fn on_spawn(&mut self, entity: Entity, _spawn_id: SpawnId, mob_type: replication::MobType) {
        if let Some(callback) = self.on_spawn {
            unsafe {
                callback(entity.to_bits(), mob_type.0);
            }
        }
    }

    fn on_component_added(
        &mut self,
        entity: Entity,
        spawn_id: SpawnId,
        component_type: replication::ComponentType,
        replicated_id: replication::Id,
    ) {
        if let Some(callback) = self.on_component_added {
            unsafe {
                callback(
                    entity.to_bits(),
                    spawn_id.0,
                    component_type.0,
                    replicated_id.0,
                );
            }
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mmoss_client_world_update(
    world: *mut WorldPtr,
    on_spawn: Option<unsafe extern "C" fn(entity: u64, mob_type: u32)>,
    on_component_updated: Option<unsafe extern "C" fn(entity: u64, id: u32)>,
    on_component_added: Option<
        unsafe extern "C" fn(entity: u64, spawn_id: u32, component_type: u32, id: u32),
    >,
) {
    if world.is_null() {
        error!("Null world passed to client_world_update");
        return;
    }

    let mut callbacks = ClientWorldUpdateCallbacks {
        on_spawn: on_spawn,
        on_component_updated,
        on_component_added,
    };

    let world = unsafe { &mut *(world as *mut WorldObj) };
    world.rtt.block_on(async {
        world
            .replication_manager
            .update_world(&mut world.bevy_world, &mut callbacks)
            .await
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn mmoss_dynamic_actor_proxy_get_tranform(
    world: *mut WorldPtr,
    entity: u64,
    out_translation: *mut Vec3,
    out_rotation: *mut Quat,
) {
    if world.is_null() {
        error!("Null world passed to mmoss_dynamic_actor_proxy_get_tranform");
        return;
    }

    let world = unsafe { &mut *(world as *mut WorldObj) };
    let entity = bevy::ecs::entity::Entity::from_bits(entity);
    if let Some(proxy) = world
        .world()
        .get::<DynamicActorComponentProxy>(entity)
    {
        let out_rotation = unsafe { &mut *out_rotation };
        let out_translation = unsafe { &mut *out_translation };
        *out_rotation = proxy.transform.rotation.into();
        *out_translation = proxy.transform.translation.into();
    }
}
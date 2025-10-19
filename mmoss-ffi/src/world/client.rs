use std::{
    ffi::{CStr, c_char},
    sync::Arc,
};

use bevy_trait_query::RegisterExt as _;
use log::error;
use mmoss::{
    core, net,
    physics::{TransformComponent, proxy::DynamicActorComponentProxy},
    replication::{self, MessageFactoryNew, Replicated},
};

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

pub struct FactoryBuilderObj {
    pub factory: Box<replication::client::Factory<bevy::ecs::world::World>>,
}

#[repr(C)]
pub struct FactoryBuilderPtr;

#[unsafe(no_mangle)]
pub extern "C" fn mmoss_client_factory_builder_new() -> *mut FactoryBuilderPtr {
    let factory = FactoryBuilderObj {
        factory: Box::new(replication::client::Factory::new()),
    };

    Box::into_raw(Box::new(factory)) as *mut FactoryBuilderPtr
}

#[unsafe(no_mangle)]
pub extern "C" fn mmoss_client_factory_builder_build(
    builder: *mut FactoryBuilderPtr,
) -> *mut FactoryPtr {
    if builder.is_null() {
        error!("Null builder passed to client_factory_builder_build");
        return std::ptr::null_mut();
    }

    let builder = unsafe { Box::from_raw(builder as *mut FactoryBuilderObj) };
    let factory = FactoryObj {
        factory: Arc::new(*builder.factory),
    };

    Box::into_raw(Box::new(factory)) as *mut FactoryPtr
}

pub struct FactoryObj {
    pub factory: Arc<replication::client::Factory<bevy::ecs::world::World>>,
}

#[repr(C)]
pub struct WorldPtr;

#[repr(C)]
pub struct FactoryPtr;

#[unsafe(no_mangle)]
pub extern "C" fn client_world_new(
    factory: *const FactoryPtr,
    addr: *const c_char,
) -> *mut WorldPtr {
    if factory.is_null() {
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

    let factory = unsafe { &*(factory as *const FactoryObj) };
    let (replication_manager, incoming) =
        replication::client::Manager::new(Box::new(transport.unwrap()), factory.factory.clone());

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
pub extern "C" fn client_world_destroy(world: *mut WorldPtr) {
    if world.is_null() {
        return;
    }

    unsafe {
        let _dropped = Box::from_raw(world as *mut WorldObj);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn client_world_update(world: *mut WorldPtr) {
    if world.is_null() {
        error!("Null world passed to client_world_update");
        return;
    }

    let world = unsafe { &mut *(world as *mut WorldObj) };
    world.rtt.block_on(async {
        world
            .replication_manager
            .update_world(&mut world.bevy_world)
            .await
    });
}

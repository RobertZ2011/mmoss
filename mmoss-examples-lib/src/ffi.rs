use bevy_trait_query::RegisterExt as _;
use log::error;
use mmoss::replication::Replicated;
use mmoss_ffi as ffi;

use crate::{
    RenderComponent, mob::{SQUARE_TYPE, SquareClient}, register_factory_components
};

#[unsafe(no_mangle)]
pub extern "C" fn mmoss_examples_lib_register_square_client(
    factory: *mut ffi::world::client::MobFactoryBuilderPtr,
) {
    if factory.is_null() {
        error!("Null factory passed to mmoss_examples_lib_register_square_client");
        return;
    }

    let factory = unsafe { &mut *(factory as *mut ffi::world::client::MobFactoryBuilderObj) };

    factory.mob_factory.register_mob(SQUARE_TYPE, SquareClient);
}

#[unsafe(no_mangle)]
pub extern "C" fn mmoss_examples_lib_world_register_components(
    world: *mut ffi::world::client::WorldPtr,
) {
    if world.is_null() {
        error!("Null world passed to mmoss_examples_lib_world_register_components");
        return;
    }

    let world = unsafe { &mut *(world as *mut ffi::world::client::WorldObj) };
    world
        .bevy_world
        .register_component_as::<dyn Replicated, RenderComponent>();
}

#[unsafe(no_mangle)]
pub extern "C" fn moss_examples_lib_register_factory_components(
    factory: *mut ffi::world::client::ComponentFactoryBuilderPtr,
) {
    if factory.is_null() {
        error!("Null factory passed to moss_examples_lib_register_factory_components");
        return;
    }

    let factory =
        unsafe { &mut *(factory as *mut ffi::world::client::ComponentFactoryBuilderObj) };

    register_factory_components(&mut factory.component_factory);
}

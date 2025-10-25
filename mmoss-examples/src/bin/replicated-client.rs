use bevy::ecs::world::World;
use bevy_trait_query::{One, RegisterExt};
use log::error;
use mmoss::net::transport::tcp;
use mmoss::physics::TransformComponent;
use mmoss::physics::proxy::DynamicActorComponentProxy;
use mmoss::replication::client::factory;
use mmoss::replication::client::{
    Manager, NoopUpdateCallbacks, factory::component::Factory as ComponentFactory,
    factory::mob::Factory as MobFactory,
};
use mmoss::replication::{MessageFactoryNew, Replicated};

use env_logger;
use mmoss_examples_lib::mob::SQUARE_TYPE;
use mmoss_examples_lib::{RenderComponent, register_factory_components};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let connection = tcp::Connection::connect("127.0.0.1:8080", MessageFactoryNew).await?;

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Client", 600, 600)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    let mut mob_factory = MobFactory::new();
    mob_factory.register_mob(SQUARE_TYPE, mmoss_examples_lib::mob::SquareClient);

    let mut component_factory = ComponentFactory::new();
    factory::component::register_default_factory_components(&mut component_factory);
    register_factory_components(&mut component_factory);

    let (mut manager, mut incoming) = Manager::new(
        Box::new(connection),
        Arc::new(mob_factory),
        Arc::new(component_factory),
    );

    tokio::spawn(async move {
        loop {
            if let Err(e) = incoming.process_incoming().await {
                error!("Error processing incoming messages: {}", e);
            }
        }
    });

    let mut world = World::new();
    world.register_component_as::<dyn TransformComponent, DynamicActorComponentProxy>();
    world.register_component_as::<dyn Replicated, DynamicActorComponentProxy>();
    world.register_component_as::<dyn Replicated, RenderComponent>();

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        manager
            .update_world(&mut world, &mut NoopUpdateCallbacks)
            .await;

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        for (render, transform) in world
            .query::<(&RenderComponent, One<&dyn TransformComponent>)>()
            .iter(&world)
        {
            render.render(&mut canvas, transform.into_inner())?;
        }
        canvas.present();
        sleep(Duration::from_secs_f32(1.0 / 30.0)).await;
    }

    Ok(())
}

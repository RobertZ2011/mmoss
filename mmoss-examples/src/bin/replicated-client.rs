use bevy::ecs::world::World;
use bevy_trait_query::{One, RegisterExt};
use mmoss::net::transport::tcp;
use mmoss::physics::TransformComponent;
use mmoss::physics::proxy::DynamicActorComponentProxy;
use mmoss::replication::client::{Factory, Manager};
use mmoss::replication::{MessageFactoryNew, Replicated};

use env_logger;
use mmoss_examples_lib::RenderComponent;
use mmoss_examples_lib::mob::SQUARE_TYPE;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let connection = tcp::Connection::connect("127.0.0.1:8080", MessageFactoryNew).await?;

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Client", 800, 600)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    let mut mob_factory = Factory::new();
    mob_factory.register_mob(SQUARE_TYPE, mmoss_examples_lib::mob::square_client);

    let (mut manager, mut incoming) = Manager::new(Box::new(connection), &mob_factory);

    tokio::spawn(async move {
        loop {
            if let Err(e) = incoming.process_incoming().await {
                eprintln!("Error processing incoming messages: {}", e);
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

        manager.update_world(&mut world).await;

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        for (render, transform) in world
            .query::<(&RenderComponent, One<&dyn TransformComponent>)>()
            .iter(&world)
        {
            render.render(&mut canvas, transform.into_inner())?;
        }
        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    Ok(())
}

use bevy::ecs::world::World;
use bevy_trait_query::RegisterExt;
use mmoss::net::transport::tcp;
use mmoss::replication::client::Manager;
use mmoss::replication::{Id, MessageFactoryNew, Replicated};

use env_logger;
use mmoss_examples_lib::ReplicatedComponent;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use static_cell::StaticCell;
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

    static MANAGER: StaticCell<Manager> = StaticCell::new();
    let manager = MANAGER.init(Manager::new(Box::new(connection)));

    tokio::spawn(async {
        loop {
            if let Err(e) = manager.process_incoming().await {
                eprintln!("Error processing incoming messages: {}", e);
            }
        }
    });

    let mut world = World::new();
    world.register_component_as::<dyn Replicated, ReplicatedComponent>();

    world.spawn(ReplicatedComponent::new(Id(0)));

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

        canvas.set_draw_color(Color::RGB(0, 255, 255));
        for replicated in world.query::<&ReplicatedComponent>().iter(&world) {
            canvas.draw_line(
                replicated.replicated.position,
                (
                    (50.0 * replicated.replicated.rotation.to_radians().cos()) as i32
                        + replicated.replicated.position.0,
                    (50.0 * replicated.replicated.rotation.to_radians().sin()) as i32
                        + replicated.replicated.position.1,
                ),
            )?;
        }
        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    Ok(())
}

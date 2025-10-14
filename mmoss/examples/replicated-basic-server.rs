use bevy::ecs::world::World;
use bevy_trait_query::RegisterExt;
use log::info;
use mmoss::{
    net::transport::tcp,
    replication::{Id, MessageFactoryNew, Replicated, server::Manager},
};
use mmoss_examples_lib::{
    RenderComponent, ReplicatedComponent, ReplicatedData, mob::square_server,
};

use rand::Rng;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Server", 800, 600)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    // Wait for connection
    let tcp = tcp::Listener::bind("127.0.0.1:8080").await.unwrap();
    let (connection, addr) = tcp.accept(MessageFactoryNew).await.unwrap();
    info!("Client connected from {}", addr);

    let mut world = World::new();
    world.register_component_as::<dyn Replicated, ReplicatedComponent>();
    world.register_component_as::<dyn Replicated, RenderComponent>();

    let mut manager = Manager::new();
    manager.add_client(Box::new(connection)).await;

    let mut id: Id = Id(1);
    let mut rng = rand::rng();

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::MouseButtonDown { x, y, .. } => {
                    let id0 = id;
                    id.0 += 1;
                    let id1 = id;
                    id.0 += 1;

                    let entity = square_server(
                        &mut world,
                        (
                            id0,
                            ReplicatedData {
                                position: (x, y),
                                rotation: 360.0 * rng.random::<f32>(),
                            },
                        ),
                        (
                            id1,
                            (rng.random::<u8>(), rng.random::<u8>(), rng.random::<u8>()),
                        ),
                    )
                    .unwrap();

                    manager.register_new_entity(entity).await;
                }
                _ => {}
            }
        }

        manager.serialize(&mut world).await;

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        for (render, replicated) in world
            .query::<(&RenderComponent, &ReplicatedComponent)>()
            .iter(&world)
        {
            canvas.set_draw_color(render.color);
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

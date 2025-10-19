use bevy::ecs::world::World;
use bevy_trait_query::{One, RegisterExt};
use log::info;
use mmoss::{
    net::transport::tcp,
    physics::{Transform, TransformComponent, proxy::DynamicActorComponentProxy},
    replication::{Id, MessageFactoryNew, Replicated, server::Manager},
};
use mmoss_examples_lib::{
    RenderComponent,
    mob::{SQUARE_TYPE, square_server_no_physics},
};

use rand::Rng;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Server", 600, 600)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    // Wait for connection
    let tcp = tcp::Listener::bind("127.0.0.1:8080").await.unwrap();
    let (connection, addr) = tcp.accept(MessageFactoryNew).await.unwrap();
    info!("Client connected from {}", addr);

    let mut world = World::new();
    world.register_component_as::<dyn TransformComponent, DynamicActorComponentProxy>();
    world.register_component_as::<dyn Replicated, DynamicActorComponentProxy>();
    world.register_component_as::<dyn Replicated, RenderComponent>();

    let mut manager = Manager::new();
    manager.add_client(Box::new(connection));

    let mut rng = rand::rng();
    let mut render_component = RenderComponent::new(Id(2));
    render_component.color = (rng.random::<u8>(), rng.random::<u8>(), rng.random::<u8>());

    let mouse_entity = world
        .spawn((
            SQUARE_TYPE,
            DynamicActorComponentProxy::new(Id(1)),
            render_component,
        ))
        .id();
    manager.register_new_entity(mouse_entity);

    let mut id: Id = Id(3);

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

                    let entity = square_server_no_physics(
                        &mut world,
                        (
                            id0,
                            Transform {
                                translation: bevy::math::Vec3::new(x as f32, y as f32, 0.0),
                                ..Default::default()
                            },
                        ),
                        (
                            id1,
                            (rng.random::<u8>(), rng.random::<u8>(), rng.random::<u8>()),
                        ),
                    )
                    .unwrap();

                    manager.register_new_entity(entity);
                }
                Event::MouseMotion { x, y, .. } => {
                    if let Ok(mut transform) = world
                        .query::<&mut DynamicActorComponentProxy>()
                        .get_mut(&mut world, mouse_entity)
                    {
                        transform.transform.translation =
                            bevy::math::Vec3::new(x as f32, y as f32, 0.0);
                    }
                    manager.mark_dirty(mouse_entity);
                }
                _ => {}
            }
        }

        manager.serialize(&mut world).await;

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

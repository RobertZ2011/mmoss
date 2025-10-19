use bevy::{ecs::entity::EntityHashSet, math::Vec3};
use bevy_trait_query::{One, RegisterExt as _};

use log::info;
use mmoss::{
    net::transport::tcp,
    physics::{self, *},
    replication::{Id, MessageFactoryNew, Replicated, server::Manager},
};
use mmoss_examples_lib::{RenderComponent, mob::square_server};
use mmoss_middleware_physx::{self as physx, CombinedWorld};

use rand::Rng as _;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let mut bevy_world = bevy::ecs::world::World::new();
    bevy_world
        .register_component_as::<dyn physics::StaticActorComponent, physx::StaticActorComponent>();
    bevy_world
        .register_component_as::<dyn physics::TransformComponent, physx::StaticActorComponent>();
    bevy_world.register_component_as::<dyn Replicated, physx::StaticActorComponent>();

    bevy_world
        .register_component_as::<dyn physics::TransformComponent, physx::DynamicActorComponent>();
    bevy_world
        .register_component_as::<dyn physics::DynamicActorComponent, physx::DynamicActorComponent>(
        );
    bevy_world.register_component_as::<dyn Replicated, physx::DynamicActorComponent>();
    bevy_world.register_component_as::<dyn Replicated, RenderComponent>();

    let mut engine = physx::Engine::new(::physx::foundation::DefaultAllocator);

    let material = Material {
        static_friction: 0.5,
        dynamic_friction: 0.5,
        restitution: 0.6,
    };

    let mut physics_world = engine.create_world(Vec3::new(0.0, -98.1, 0.0)).await?;
    let mut plane_entity = bevy_world.spawn_empty();
    let plane_component = physics_world
        .create_plane(
            plane_entity.id(),
            Id(1),
            &material,
            &PlaneShape {
                normal: Vec3::Y,
                offset: 0.0,
            },
        )
        .await?;
    plane_entity.insert(plane_component);

    let mut world = CombinedWorld {
        bevy_world,
        physics_world,
    };

    // Wait for connection
    let tcp = tcp::Listener::bind("127.0.0.1:8080").await.unwrap();
    let (connection, addr) = tcp.accept(MessageFactoryNew).await.unwrap();
    info!("Client connected from {}", addr);

    let mut manager = Manager::new();
    manager.add_client(Box::new(connection));

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Server", 600, 600)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    let mut id: Id = Id(1);
    let mut rng = rand::rng();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut to_replicate = EntityHashSet::new();
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
                    .await?;

                    manager.register_new_entity(entity);
                    to_replicate.insert(entity);
                }
                _ => {}
            }
        }

        for entity in &to_replicate {
            manager.mark_dirty(*entity);
        }

        world
            .physics_world
            .update_world(&mut world.bevy_world, 1.0 / 30.0)?;
        let now = std::time::Instant::now();
        manager.serialize(&mut world.bevy_world).await;
        info!(
            "Serialization took {:?} for {} entities",
            now.elapsed(),
            to_replicate.len()
        );

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        canvas.set_draw_color(Color::RGB(255, 0, 0));

        let mut query = world
            .bevy_world
            .query::<(&RenderComponent, One<&dyn TransformComponent>)>();
        for (render, transform) in query.iter(&world.bevy_world) {
            render.render(&mut canvas, transform.into_inner())?;
        }

        canvas.present();
        sleep(Duration::from_secs_f32(1.0 / 30.0)).await;
    }
    Ok(())
}

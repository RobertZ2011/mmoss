use bevy::math::Vec3;
use bevy_trait_query::{One, RegisterExt as _};

use mmoss::{
    physics::{self, *},
    replication::Id,
};
use mmoss_middleware_physx as physx;

use anyhow::anyhow;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::{event::Event, rect::Rect};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let mut bevy_world = bevy::ecs::world::World::new();
    bevy_world
        .register_component_as::<dyn physics::StaticActorComponent, physx::StaticActorComponent>();
    bevy_world
        .register_component_as::<dyn physics::DynamicActorComponent, physx::DynamicActorComponent>(
        );

    let mut engine = physx::Engine::new(::physx::foundation::DefaultAllocator);

    let mut world = engine.create_world(Vec3::new(0.0, -9.81, 0.0)).await?;

    let material = Material {
        static_friction: 0.5,
        dynamic_friction: 0.5,
        restitution: 0.6,
    };
    let mut plane_entity = bevy_world.spawn_empty();
    let plane_component = world
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

    let mut sphere_entity = bevy_world.spawn_empty();
    let sphere_component = world
        .create_dynamic_actor_component(
            sphere_entity.id(),
            Id(2),
            &Transform {
                position: Vec3::new(0.0, 10.0, 0.0),
                ..Default::default()
            },
            10.0,
            &material,
            &[(
                Shape::Sphere(SphereShape { radius: 1.0 }),
                Transform::default(),
            )],
        )
        .await?;
    sphere_entity.insert(sphere_component);

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Client", 800, 600)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

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

        world.update_world(&mut bevy_world, 1.0 / 60.0)?;
        let mut query = bevy_world.query::<One<&dyn physics::DynamicActorComponent>>();

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        canvas.set_draw_color(Color::RGB(255, 0, 0));
        for actor in query.iter(&bevy_world) {
            let transform = actor.transform();
            canvas
                .draw_rect(Rect::from_center(
                    (
                        (transform.position.x * 10.0 + 100.0) as i32,
                        100 - (transform.position.y * 10.0) as i32,
                    ),
                    10,
                    10,
                ))
                .map_err(|e| anyhow!("Failed to draw rect: {}", e))?;
        }

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
    Ok(())
}

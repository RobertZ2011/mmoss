use std::net::SocketAddr;

use mmoss::{
    net::transport::{Addressed, Unreliable, VecU8FactoryNew},
    replication::{Id, Replicated},
};
use mmoss_examples_lib::{ReplicatedData, Square};

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut udp = mmoss::net::transport::udp::Udp::bind("0.0.0.0:0", VecU8FactoryNew).await?;
    let address: SocketAddr = "127.0.0.1:8080".parse().unwrap();

    let mut square = Square {
        id: Id(0),
        replicated: ReplicatedData {
            rotation: 45.7,
            position: (250, 100),
        },
    };

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Server", 800, 600)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(0, 255, 255));
    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::MouseButtonDown { .. } => {
                    square.replicated.rotation += 45.0;
                    square.replicated.rotation %= 360.0;

                    let mut vec = Vec::with_capacity(512);
                    square.serialize(&mut vec)?;
                    let message = Addressed {
                        message: vec,
                        address,
                    };

                    udp.send(message).await?;
                }
                _ => {}
            }
        }

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        canvas.set_draw_color(Color::RGB(0, 255, 255));
        canvas.draw_line(
            square.replicated.position,
            (
                (50.0 * square.replicated.rotation.to_radians().cos()) as i32
                    + square.replicated.position.0,
                (50.0 * square.replicated.rotation.to_radians().sin()) as i32
                    + square.replicated.position.1,
            ),
        )?;
        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    Ok(())
}

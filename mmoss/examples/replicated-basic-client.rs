use mmoss::net::transport::{Unreliable, VecU8FactoryNew};
use mmoss::replication::{Id, Replicated};

use mmoss_examples_lib::{ReplicatedData, Square};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut udp = mmoss::net::transport::udp::Udp::bind("127.0.0.1:8080", VecU8FactoryNew).await?;

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Client", 800, 600)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    let mut square = Square {
        id: Id(0),
        replicated: ReplicatedData {
            rotation: 0.0,
            position: (0, 0),
        },
    };

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        if let Ok(Some(message)) = udp.try_receive() {
            let data: Vec<u8> = message.message;
            println!("Received message from {}: {:?}", message.address, data);

            let mut cursor = std::io::Cursor::new(data);
            square.replicate(&mut cursor).unwrap();
        }

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

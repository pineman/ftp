#![feature(array_chunks)]

use std::fs;

use clap::Parser;

use winit::{
    dpi::LogicalSize,
    event::{Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};

use fpt::Gameboy;
use pixels::{Pixels, SurfaceTexture};

const GB_RESOLUTION: (u32, u32) = (160, 144);
const SCALE: u32 = 3;
const PALETTE: [[u8; 4]; 4] = [
    [0, 63, 0, 255],
    [46, 115, 32, 255],
    [140, 191, 10, 255],
    [160, 207, 10, 255],
];

const FRAME_IN_M_CYCLES: u32 = 17556;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Flag to active debug output
    #[arg(short, long)]
    debug: bool,
}

fn main() -> Result<(), pixels::Error> {
    let args = Args::parse();


    let ctx = zmq::Context::new();
    let socket = ctx.socket(zmq::REP).unwrap();
    socket.bind("tcp://127.0.0.1:5000");

    let event_loop: EventLoop<()> = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("fpt (winit + pixels)")
        .with_inner_size(LogicalSize::new(
            SCALE * GB_RESOLUTION.0,
            SCALE * GB_RESOLUTION.1,
        ))
        .with_min_inner_size(LogicalSize::new(GB_RESOLUTION.0, GB_RESOLUTION.1))
        .build(&event_loop)
        .unwrap();

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(GB_RESOLUTION.0, GB_RESOLUTION.1, surface_texture)?
    };

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            event:
                ref e @ (WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                    ..
                }),
            ..
        } => {
            println!(
                "{reason}; stopping",
                reason = match e {
                    WindowEvent::CloseRequested => "The close button was pressed",
                    WindowEvent::KeyboardInput { .. } => "The ESC key was pressed",
                    _ => "whatever",
                }
            );
            control_flow.set_exit();
        }
        Event::WindowEvent {
            event: WindowEvent::Resized(size),
            ..
        } => {
            if let Err(err) = pixels.resize_surface(size.width, size.height) {
                eprintln!("pixels.resize_surface() error! {err}");
                control_flow.set_exit_with_code(1);
                return;
            }
        }
        Event::MainEventsCleared => {
            println!("wait");
            let mut message: zmq::Message = zmq::Message::new();
            socket.recv(&mut message, 0);
            println!("after");
            dbg!(message.as_str());

            //draw(pixels.frame_mut(), the_frame);

            if let Err(err) = pixels.render() {
                eprintln!("pixels.render() error! {err}");
                control_flow.set_exit_with_code(2);
                return;
            }
            // window.request_redraw();
        }
        _ => (),
    });
}

fn draw(pixels_frame: &mut [u8], gb_frame: &fpt::ppu::Frame) {
    for (i, chunk) in pixels_frame.array_chunks_mut::<4>().enumerate() {
        chunk.copy_from_slice(&PALETTE[gb_frame[i] as usize]);
    }
}

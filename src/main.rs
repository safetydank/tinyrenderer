#![deny(clippy::all)]
#![forbid(unsafe_code)]

use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

use std::env;
use std::fs;

mod renderer;
pub use renderer::Renderer;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 800;

struct Vertex {
   x: f32,
   y: f32,
   z: f32,
}

impl Vertex {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

fn main() -> Result<(), Error> {
    env_logger::init();
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("tinyrenderer")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };

    
    let mut renderer = Renderer::new(WIDTH, HEIGHT);

    // hacking an obj reader with no error handling
    let content = fs::read_to_string("obj/african_head.obj").expect("Error reading file");
    let lines = content.split("\n");

    let mut vertices = vec![];
    let mut vindexes = vec![];

    vertices.push(Vertex::new(0.0, 0.0, 0.0));

    for line in lines {
        if line.starts_with("v ") {
            let mut values = line.split(" ").skip(1);
            let x = values.next().unwrap().parse::<f32>().unwrap();
            let y = values.next().unwrap().parse::<f32>().unwrap();
            let z = values.next().unwrap().parse::<f32>().unwrap();
            vertices.push(Vertex::new(x, y, z));
            println!("Pushed x {} y {} z {}", x, y, z);
        } else if line.starts_with("f ") {
            let mut values = line.split(" ").skip(1);
            for _ in 0..3 {
                let index = values.next().unwrap().split("/").next().unwrap().parse::<i32>().unwrap();
                vindexes.push(index);
            }
        }
    }
    
    for tri in vindexes.chunks_exact_mut(3) {
        let w = renderer.width as f32;
        let h = renderer.height as f32;
        println!("Triangle {} {} {}", tri[0], tri[1], tri[2]);
        for j in 0..3 {
            let i0 = tri[j] as usize;
            let i1 = tri[(j+1) % 3] as usize;
            let v0 = &vertices[i0];
            let v1 = &vertices[i1];
            let x0 = (v0.x + 1.0) * w / 2.0;
            let y0 = (v0.y + 1.0) * h / 2.0;
            let x1 = (v1.x + 1.0) * w / 2.0;
            let y1 = (v1.y + 1.0) * h / 2.0;
            println!("Render x0 {} y0 {} x1 {} y1 {}", x0, y0, x1, y1);
            renderer.line(x0 as i32, y0 as i32, x1 as i32, y1 as i32, 0xffffffff);
        }
    }


    event_loop.run(move |event, _, control_flow| {
        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            renderer.draw(pixels.get_frame());
            if pixels
                .render()
                .map_err(|e| error!("pixels.render() failed: {}", e))
                .is_err()
            {
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        // Handle input events
        if input.update(&event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                pixels.resize_surface(size.width, size.height);
            }

            // Update internal state and request a redraw
            // XXX .update()
            window.request_redraw();
        }
    });
}



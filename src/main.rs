#![deny(clippy::all)]
#![forbid(unsafe_code)]

use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

use std::mem;

const WIDTH: u32 = 640;
const HEIGHT: u32 = 480;

struct Renderer {
    width: u32,
    height: u32,
    buf: Vec<u32>
}

impl Renderer {
    fn new(width: u32, height: u32) -> Self {
        Self {
            width: width,
            height: height,
            buf: vec![0; (width * height) as usize],
        }
    }
    
    fn pixel(&mut self, x: u32, y: u32, color: u32) {
        let offset = ((self.height - y - 1) * self.width + x) as usize;
        self.buf[offset] = color;
    }
    
    fn line(&mut self, mut x0: i32, mut y0: i32, mut x1: i32, mut y1: i32, color: u32) {
        let steep = (x0-x1).abs() < (y0-y1).abs();
        if steep {
            mem::swap(&mut x0, &mut y0);
            mem::swap(&mut x1, &mut y1);
        }
        
        if x0 > x1 {
            mem::swap(&mut x0, &mut x1);
            mem::swap(&mut y0, &mut y1);
        }
        
        let dx = (x1-x0) as f32;
        let dy = (y1-y0) as f32;
        let derror = (dy/dx).abs();
        let mut error = 0.0;
        let mut y = y0;
        for x in x0..x1 {
            if steep {
                self.pixel(y as u32, x as u32, color)
            } else {
                self.pixel(x as u32, y as u32, color)
            }
            error += derror;
            if error > 0.5 {
                y += if y1 > y0 { 1 } else { -1 };
                error -= 1.0;
            }
        }
    }

    fn draw(&self, frame: &mut [u8]) {
        for (b, p) in self.buf.iter().zip(frame.chunks_exact_mut(4)) {
            p.copy_from_slice(&b.to_be_bytes());
        }
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
    renderer.line(20, 13, 40, 80, 0xff0000ff); 
    renderer.line(80, 40, 13, 20, 0xff0000ff);

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



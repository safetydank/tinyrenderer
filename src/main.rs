#![deny(clippy::all)]
#![forbid(unsafe_code)]

use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

use std::path::Path;
use std::fs::File;
use std::io::BufWriter;

mod renderer;
pub use renderer::Renderer;

mod objloader;
pub use objloader::{load_obj, Mesh};

const WIDTH: i32 = 800;
const HEIGHT: i32 = 800;

pub fn save_png(path_str: &str, width: u32, height: u32, buf: &[u32]) {
    let path = Path::new(&path_str);
    let file = File::create(path).unwrap();
    let ref mut w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);

    let mut writer = encoder.write_header().unwrap();
    // convert u32 buffer to u8
    let bbuf:Vec<u8> = buf.iter().flat_map(|v| v.to_be_bytes()).collect();
    writer.write_image_data(&bbuf).unwrap(); // Save
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
        Pixels::new(WIDTH as u32, HEIGHT as u32, surface_texture)?
    };

    let mut renderer = Renderer::new(WIDTH, HEIGHT);

    let mesh = objloader::load_obj("obj/african_head.obj");

    for tri in mesh.vis.chunks_exact(3) {
        let w = (renderer.width - 1) as f32;
        let h = (renderer.height - 1) as f32;
        println!("Triangle {} {} {}", tri[0], tri[1], tri[2]);
        for j in 0..3 {
            let i0 = tri[j] as usize;
            let i1 = tri[(j+1) % 3] as usize;
            let v0 = &mesh.vs[i0];
            let v1 = &mesh.vs[i1];
            let x0 = (v0.x + 1.0) * w / 2.0;
            let y0 = (v0.y + 1.0) * h / 2.0;
            let x1 = (v1.x + 1.0) * w / 2.0;
            let y1 = (v1.y + 1.0) * h / 2.0;
            println!("Render x0 {} y0 {} x1 {} y1 {}", x0, y0, x1, y1);
            renderer.line(x0 as i32, y0 as i32, x1 as i32, y1 as i32, 0xffffffff);
        }
    }

    save_png("wires.png", renderer.width as u32, renderer.height as u32, renderer.buf.as_slice());

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



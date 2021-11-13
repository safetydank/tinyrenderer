#![deny(clippy::all)]
#![forbid(unsafe_code)]

use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;
use rand;

use std::path::Path;
use std::fs::File;
use std::io::BufWriter;

mod renderer;
mod objloader;
mod geometry;

use renderer::Renderer;
use objloader::load_obj;
use geometry::{Vec2i, Vec3f, cross, dot};

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

fn draw(r: &mut Renderer) {
    let mesh = load_obj("obj/african_head.obj");
    let mut rng = rand::thread_rng();

    let light_dir = Vec3f::new(0.0, 0.0, -1.0);

    for tri in mesh.vis.chunks_exact(3) {
        let w = (r.width - 1) as f32;
        let h = (r.height - 1) as f32;
        // println!("Triangle {} {} {}", tri[0], tri[1], tri[2]);
        
        // world space vertices
        let vs: Vec<Vec3f> = tri.iter().map(|i| {
            mesh.vs[*i as usize]
        }).collect();

        // project vertices into screen space points
        let pts: Vec<Vec2i> = vs.iter().map(|v| {
            Vec2i{
                x: ((v.x + 1.0) * w / 2.0) as i32,
                y: ((v.y + 1.0) * h / 2.0) as i32,
            }
        }).collect();
        
        // normal
        let n = cross(vs[2].sub(vs[0]), vs[1].sub(vs[0])).normalized();
        let intensity = (dot(n, light_dir) * 255.0) as u32;

        // let color = rng.gen::<u32>() | 0xff;
        // r.triangle(pts[0], pts[1], pts[2], color);
        if intensity > 0 {
            let color = (intensity<<24) | (intensity<<16) | (intensity<<8) | 0xff;
            r.triangle_fill(pts, color);
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
        Pixels::new(WIDTH as u32, HEIGHT as u32, surface_texture)?
    };

    let mut renderer = Renderer::new(WIDTH, HEIGHT);
    draw(&mut renderer);

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



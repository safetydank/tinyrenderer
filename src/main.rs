#![deny(clippy::all)]
#![forbid(unsafe_code)]

use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

use tinyrenderer::renderer::{Renderer, Texture};
use tinyrenderer::objloader::{load_obj, Mesh};
use tinyrenderer::util::{draw_mesh, load_png_texture, save_png};

use glam::Vec3A;

const WIDTH: i32 = 800;
const HEIGHT: i32 = 800;

fn draw(r: &mut Renderer, tex: &Texture) {
    let mesh: Mesh<Vec3A> = load_obj("obj/african_head.obj");
    draw_mesh(r, &mesh, tex);
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
    let texture = load_png_texture("obj/african_head_diffuse.png");
    texture.log_debug();
    draw(&mut renderer, &texture);

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



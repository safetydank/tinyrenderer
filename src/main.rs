#![deny(clippy::all)]
#![forbid(unsafe_code)]

use egui::TextBuffer;
use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use tinyrenderer::geometry::Vector3;
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

use tinyrenderer::renderer::{Renderer, Texture, RendererState};
use tinyrenderer::objloader::{load_obj};
use tinyrenderer::util::{load_png_texture, save_png};
use tinyrenderer::gui::Framework;

const WIDTH: i32 = 1000;
const HEIGHT: i32 = 1000;

fn draw(r: &mut Renderer, s: &RendererState) {
    let mesh = load_obj("obj/african_head.obj");
    let diffuse = load_png_texture("obj/african_head_diffuse.png");
    let normal= load_png_texture("obj/african_head_nm_tangent.png");
    // let normal= load_png_texture("obj/african_head_nm.png");

    diffuse.log_debug();
    normal.log_debug();
    // r.draw_mesh(&mesh, tex);
    r.draw_mesh_shader(s, &mesh, &diffuse, &normal);
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

    let (mut pixels, mut framework) = {
        let window_size = window.inner_size();
        let scale_factor = window.scale_factor() as f32;
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        let pixels = Pixels::new(WIDTH as u32, HEIGHT as u32, surface_texture)?;
        let framework = Framework::new(window_size.width, window_size.height, scale_factor, &pixels);

        (pixels, framework)
    };

    let mut renderer = Renderer::new(WIDTH, HEIGHT);
    // let mut renderer_state = RendererState{
    //     model: Vector3::ZERO,
    //     eye: Vector3::new(1.0, 1.0, 3.0),
    //     center: Vector3::ZERO,
    //     up: Vector3::new(0.0, 1.0, 0.0),
    //     light_dir: Vector3::new(1.0, 1.0, 1.0),
    //     rotation: Vector3::ZERO,
    // };
    // renderer.clear();
    draw(&mut renderer, &framework.gui.renderer_state);

    save_png("shaded.png", renderer.width as u32, renderer.height as u32, renderer.buf.as_slice());
    save_png("zbuf.png", renderer.width as u32, renderer.height as u32, renderer.zbuf_buf().as_slice());


    event_loop.run(move |event, _, control_flow| {
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
        
        match event {
            Event::WindowEvent { event, .. } => {
                // Update egui inputs
                if framework.handle_event(&event) {
                    renderer.clear();
                    draw(&mut renderer, &framework.gui.renderer_state);
                }
            }
            Event::RedrawRequested(_) => {
                renderer.draw(pixels.get_frame());
                framework.prepare(&window);
                let render_result = pixels.render_with(|encoder, render_target, context| {
                    // Render the world texture
                    context.scaling_renderer.render(encoder, render_target);

                    // Render egui
                    framework.render(encoder, render_target, context)?;

                    Ok(())
                });

                // Basic error handling
                if render_result
                    .map_err(|e| error!("pixels.render() failed: {}", e))
                    .is_err()
                {
                    *control_flow = ControlFlow::Exit;
                }
            }
            _ => (),
        }
    });
}



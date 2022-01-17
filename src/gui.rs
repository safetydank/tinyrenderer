use egui::{ClippedMesh, CtxRef, Ui};
use egui_wgpu_backend::{BackendError, RenderPass, ScreenDescriptor};
use pixels::{wgpu, PixelsContext};
use winit::window::Window;

use crate::{geometry::Vector3, renderer::RendererState};

/// Manages all state required for rendering egui over `Pixels`.
pub struct Framework {
    // State for egui.
    egui_ctx: CtxRef,
    egui_state: egui_winit::State,
    screen_descriptor: ScreenDescriptor,
    rpass: RenderPass,
    paint_jobs: Vec<ClippedMesh>,

    // State for the GUI
    pub gui: Gui,
}

pub struct Gui {
    /// Only show the egui window when true.
    window_open: bool,
    
    pub renderer_state: RendererState,
}

impl Framework {
    /// Create egui.
    pub fn new(width: u32, height: u32, scale_factor: f32, pixels: &pixels::Pixels) -> Self {
        let egui_ctx = CtxRef::default();
        let egui_state = egui_winit::State::from_pixels_per_point(scale_factor);
        let screen_descriptor = ScreenDescriptor {
            physical_width: width,
            physical_height: height,
            scale_factor,
        };
        let rpass = RenderPass::new(pixels.device(), pixels.render_texture_format(), 1);
        let gui = Gui::new();

        Self {
            egui_ctx,
            egui_state,
            screen_descriptor,
            rpass,
            paint_jobs: Vec::new(),
            gui,
        }
    }

    /// Handle input events from the window manager.
    pub fn handle_event(&mut self, event: &winit::event::WindowEvent) -> bool {
        self.egui_state.on_event(&self.egui_ctx, event)
    }

    /// Resize egui.
    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.screen_descriptor.physical_width = width;
            self.screen_descriptor.physical_height = height;
        }
    }

    /// Update scaling factor.
    pub fn scale_factor(&mut self, scale_factor: f64) {
        self.screen_descriptor.scale_factor = scale_factor as f32;
    }

    /// Prepare egui.
    pub fn prepare(&mut self, window: &Window) {
        // Begin the egui frame.
        let raw_input = self.egui_state.take_egui_input(window);
        self.egui_ctx.begin_frame(raw_input);

        // Draw the demo application.
        self.gui.ui(&self.egui_ctx);

        // End the egui frame and create all paint jobs to prepare for rendering.
        let (output, paint_commands) = self.egui_ctx.end_frame();
        self.egui_state
            .handle_output(window, &self.egui_ctx, output);
        self.paint_jobs = self.egui_ctx.tessellate(paint_commands);
    }

    /// Render egui.
    pub fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        render_target: &wgpu::TextureView,
        context: &PixelsContext,
    ) -> Result<(), BackendError> {
        // Upload all resources to the GPU.
        self.rpass
            .update_texture(&context.device, &context.queue, &self.egui_ctx.texture());
        self.rpass
            .update_user_textures(&context.device, &context.queue);
        self.rpass.update_buffers(
            &context.device,
            &context.queue,
            &self.paint_jobs,
            &self.screen_descriptor,
        );

        // Record all render passes.
        self.rpass.execute(
            encoder,
            render_target,
            &self.paint_jobs,
            &self.screen_descriptor,
            None,
        )
    }
}

impl Gui {
    /// Create a `Gui`.
    fn new() -> Self {
        Self {
            window_open: true,
            renderer_state: RendererState{
                model: Vector3::ZERO,
                eye: Vector3::new(1.0, 1.0, 3.0),
                center: Vector3::ZERO,
                up: Vector3::new(0.0, 1.0, 0.0),
                light_dir: Vector3::new(1.0, 1.0, 1.0),
                rotation: Vector3::ZERO,
            }
        }
    }

    /// Create the UI using egui.
    fn ui(&mut self, ctx: &CtxRef) {
        let RendererState {
            model,
            eye,
            center,
            up,
            light_dir,
            rotation,
        } = &mut self.renderer_state;

        egui::TopBottomPanel::top("menubar_container").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                egui::menu::menu(ui, "Controls", |ui| {
                    if ui.button("Renderer").clicked() {
                        self.window_open = true;
                    }
                })
            });
        });

        egui::Window::new("Renderer")
            .open(&mut self.window_open)
            .show(ctx, |ui| {
                egui::Grid::new("renderer").show(ui, |ui| {
                    drag_vec3_row(ui, "Model", model);
                    drag_vec3_row(ui, "Eye", eye);
                    drag_vec3_row(ui, "Center", center);
                    drag_vec3_row(ui, "Up", up);
                    drag_vec3_row(ui, "Light dir", light_dir);
                    drag_vec3_row(ui, "Rotation", rotation);
                });
            });
    }
    
}

//  A grid row with a label and 3 draggable numbers bound to a Vector3 
fn drag_vec3_row(ui: &mut Ui, label: &str, v: &mut Vector3) {
    ui.label(label);
    ui.horizontal(|ui| {
        ui.label("x");
        ui.add(egui::DragValue::new(&mut v.x).speed(1.0));
        ui.label("y");
        ui.add(egui::DragValue::new(&mut v.y).speed(1.0));
        ui.label("z");
        ui.add(egui::DragValue::new(&mut v.z).speed(1.0));
    });
    ui.end_row();
}


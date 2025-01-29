
use egui::Context;
use egui_wgpu::ScreenDescriptor;
use wgpu::SurfaceConfiguration;
use winit::window::Window;

pub struct UIFrame {
    pub clipped_primitives: Vec<egui_winit::egui::ClippedPrimitive>,
    pub textures_delta: egui::TexturesDelta
}


pub struct UIManager {
    // context: egui::Context,
    winit_state: egui_winit::State,
    screen_descriptor: egui_wgpu::ScreenDescriptor,
    ui_renderer: egui_wgpu::Renderer,
    frame: Option<UIFrame>,
}

impl UIManager {
    pub fn new(
        window: &Window,
        device: &wgpu::Device,
        surface_config: &SurfaceConfiguration,
        depth_format: Option<wgpu::TextureFormat>,
    ) -> UIManager {
        let context = egui::Context::default();
        let viewport_id = context.viewport_id();

        let winit_state = egui_winit::State::new(
            context,
            viewport_id,
            window,
            Some(1.0),
            None,
            None,
        );

        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [surface_config.width, surface_config.height],
            pixels_per_point: 1.0,
        };

        let ui_renderer = egui_wgpu::Renderer::new(
            device,
            surface_config.format,
            depth_format,
            1,
            false,
        );

        let frame = None;

        UIManager {
            // context,
            winit_state,
            screen_descriptor,
            ui_renderer,
            frame,
        }
    }

    pub fn update_resources(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        surface_config: &SurfaceConfiguration,
    ) {
        self.screen_descriptor = ScreenDescriptor {
            size_in_pixels: [surface_config.width, surface_config.height],
            pixels_per_point: 1.0,
        };

        if let Some(ui_frame) = &self.frame {
            // texture sets
            for (texture_id, image_delta) in &ui_frame.textures_delta.set {
                self.ui_renderer.update_texture(
                    device,
                    queue,
                    *texture_id,
                    image_delta,
                );
            }

            // texture frees
            for texture_id in &ui_frame.textures_delta.free {
                self.ui_renderer.free_texture(texture_id);
            }

            // update
            self.ui_renderer.update_buffers(
                device,
                queue,
                encoder,
                &ui_frame.clipped_primitives,
                &self.screen_descriptor
            );
        }
    }

    pub fn run<F: FnMut(&Context)>(&mut self, window: &Window, gui: F) {
        match self.frame {
            None => {
                let raw_input = self.winit_state.take_egui_input(window);
        
                // todo: handle viewport updates (per https://docs.rs/egui-winit/latest/egui_winit/struct.State.html#method.take_egui_input)
        
                let full_output = self.winit_state.egui_ctx().run(raw_input, gui);

                // let full_output = self.winit_state.egui_ctx().run(raw_input, |context| {
                //     egui::TopBottomPanel::bottom("bottom_panel").show(&context, |ui| {
                //         ui.label("Check out my awesome bottom panel!");
                //         if ui.button("click to print to stdout").clicked() {
                //             println!("Hello, World!");
                //         }
                //     });
                // });
        
                // handle any extra platform output 
        
                let clipped_primitives = self.winit_state.egui_ctx().tessellate(full_output.shapes, full_output.pixels_per_point);
                let textures_delta = full_output.textures_delta;
        
                self.frame = Some(UIFrame {
                    clipped_primitives,
                    textures_delta,
                });
            },
            Some(_) => {
                panic!("UIManager: called run twice before render");
            }
        }
    }

    pub fn render(
        &mut self,
        render_pass: &mut wgpu::RenderPass<'static>,
    ) {
        if let Some(frame) = &self.frame {
            self.ui_renderer.render(
                render_pass,
                &frame.clipped_primitives,
                &self.screen_descriptor,
            );
        }

        self.frame = None;
    }

    // todo: multiple windows?
    pub fn on_window_event(&mut self, window: &Window, event: &winit::event::WindowEvent) -> bool {
        let response = self.winit_state.on_window_event(window, event);
        response.consumed
    }
}
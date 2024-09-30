use crate::renderer::UIFrame;
use winit::window::Window;

pub struct UIManager {
    // context: egui::Context,
    winit_state: egui_winit::State,
}

impl UIManager {
    pub fn new(window: &Window) -> UIManager {
        let context = egui::Context::default();
        let viewport_id = context.viewport_id();

        let winit_state = egui_winit::State::new(
            context,
            viewport_id,
            window,
            Some(1.0),
            None,
        );

        UIManager {
            // context,
            winit_state,
        }
    }

    pub fn run(&mut self, window: &Window) -> UIFrame {
        let raw_input = self.winit_state.take_egui_input(window);
        // let raw_input = egui::RawInput::default();

        // todo: handle viewport updates (per https://docs.rs/egui-winit/latest/egui_winit/struct.State.html#method.take_egui_input)

        // let full_output = self.context.run(raw_input, |context| {
        //     egui::Window::new("hello").show(&context, |ui| {
        //         ui.label("Check out my awesome UI!");
        //         if ui.button("click to print to stdout").clicked() {
        //             println!("Hello, World!");
        //         }
        //     });
        // });

        let full_output = self.winit_state.egui_ctx().run(raw_input, |context| {
            egui::TopBottomPanel::bottom("bottom_panel").show(&context, |ui| {
                ui.label("Check out my awesome bottom panel!");
                if ui.button("click to print to stdout").clicked() {
                    println!("Hello, World!");
                }
            });
        });

        // handle any extra platform output 

        let clipped_primitives = self.winit_state.egui_ctx().tessellate(full_output.shapes, full_output.pixels_per_point);
        let textures_delta = full_output.textures_delta;

        UIFrame {
            clipped_primitives,
            textures_delta,
        }
    }

    // todo: multiple windows?
    pub fn on_window_event(&mut self, window: &Window, event: &winit::event::WindowEvent) -> bool {
        let response = self.winit_state.on_window_event(window, event);
        response.consumed
    }
}
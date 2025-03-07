use std::time::Duration;

use cgmath::{Vector3, Zero};
use shaderunner::{Camera, Game, GameProgram, Renderer};
use winit::{dpi::PhysicalPosition, event::WindowEvent, window::Window};

pub struct CursorGame {
    renderer: Renderer,
    camera: Camera,
    cursor_position: PhysicalPosition<f64>,
}


impl Game for CursorGame {
    async fn new(window: Window) -> Self {
        let mut renderer = Renderer::new(window).await;

        let camera = Camera::new(
            Vector3::zero(),
            Vector3::unit_y(),
            Vector3::unit_z(),
            1.25,
            std::f32::consts::TAU / 8.0,
            0.1,
            1000.0,
        );
        renderer.update_camera(&camera);

        let cursor_position = PhysicalPosition::new(0.0, 0.0);

        Self {
            renderer,
            camera,
            cursor_position,
        }
    }

    fn resize(&mut self, new_size: &winit::dpi::PhysicalSize<u32>) {
        self.camera.resize(&new_size);
        self.renderer.update_camera(&self.camera);
        self.renderer.resize(&new_size);
    }

    fn window_event(&mut self, event: &WindowEvent) -> bool {
        if let WindowEvent::CursorMoved { position, .. } = event {
            self.cursor_position = *position;
        }

        self.renderer.egui_event(event)
    }

    fn update(&mut self) {

    }
    
    fn render(&mut self, _since_render: Duration, _since_update: Duration) {
        // draw star on cursor position
        let ray = self.camera.pixel_to_ray(self.renderer.window().inner_size(), self.cursor_position.cast());
        let position = ray.source() + 10.0 * ray.direction();

        self.renderer.draw_line_green(position - Vector3::unit_x()/4_f32, position + Vector3::unit_x()/4_f32);
        self.renderer.draw_line_green(position - Vector3::unit_y()/4_f32, position + Vector3::unit_y()/4_f32);
        self.renderer.draw_line_green(position - Vector3::unit_z()/4_f32, position + Vector3::unit_z()/4_f32);

        // render
        self.renderer.render();
    }
}


#[cfg_attr(target_arch="wasm32", wasm_bindgen::prelude::wasm_bindgen(start))]
fn main() {
    pollster::block_on(
        shaderunner::window::run_program::<GameProgram<CursorGame>>()
    )
}
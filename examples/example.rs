use std::time::Duration;

use cgmath::Vector3;
use shaderunner::{game_program::GameProgram, renderer::Renderer, scene::{camera::Camera, light::{AmbientLight, DirectionalLight}, Transform}, window::Game};
use winit::{event::Event, window::Window};


pub struct ExampleGame {
    renderer: Renderer,
    camera: Camera,
}

impl ExampleGame {
}

impl Game for ExampleGame {
    async fn new(window: Window) -> ExampleGame {
        let mut renderer = Renderer::new(window).await;
    
        let cube_mesh = shaderunner::test_assets::cube_mesh();
        let cube_mesh = renderer.add_mesh(&cube_mesh);
        let _instance0 = renderer.add_instance(cube_mesh, Transform::from_translation(Vector3::new(-0.5, -0.5, -0.5)));
    
        let sphere_mesh = shaderunner::test_assets::simple_sphere_mesh(1.0, 16, Vector3::new(1.0, 1.0, 1.0));
        let sphere_mesh = renderer.add_mesh(&sphere_mesh);
        let _instance1 = renderer.add_instance(sphere_mesh, Transform::from_translation(Vector3::new(0.5, 0.5, 0.5)));
    
        // let quad_mesh = echoes_graphics::test_assets::gradient_quad_mesh();
        // let quad_mesh = renderer.add_mesh(&quad_mesh);
        // let _instance2 = renderer.add_instance(quad_mesh, Transform::identity());
    
        let ambient = AmbientLight::new(Vector3::new(1.0, 1.0, 1.0), 0.02);
        let directional = DirectionalLight::new(
            Vector3::new(1.0, -0.6, -1.0),
            Vector3::new(1.0, 1.0, 1.0),
            0.5,
        );
    
        renderer.update_light(&directional, &ambient);
    
        let camera = Camera::new(
            Vector3::new(0.8, -1.5, 1.0),
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::unit_z(),
            1.25,
            std::f32::consts::TAU / 8.0,
            0.1,
            100.0,
        );
    
        renderer.update_camera(&camera);
        
        ExampleGame {
            renderer,
            camera,
        }
    
    }

    fn resize(&mut self, new_size: &winit::dpi::PhysicalSize<u32>) {
        self.camera.resize(&new_size);
        self.renderer.update_camera(&self.camera);
        self.renderer.resize(&new_size);
    }

    fn input(&mut self, _event: &Event<()>) -> bool {
        false
    }

    fn update(&mut self) {

    }

    fn render(&mut self, _since_render: Duration, _since_update: Duration) {
        // draw a green_line in immediate mode
        self.renderer.draw_line_green(Vector3::new(-10_f32, -10_f32, -10_f32), Vector3::new(10_f32, 10_f32, 10_f32));

        // draw some UI in immediate mode
        self.renderer.run_ui(|context| {
            egui::TopBottomPanel::bottom("bottom_panel").show(&context, |ui| {
                ui.label("Check out my awesome bottom panel!");
                if ui.button("click to print to stdout").clicked() {
                    println!("Hello, World!");
                }
            });
        });

        // render
        self.renderer.render();
    }
}

#[cfg(target_arch="wasm32")]
use wasm_bindgen::prelude::*;


#[cfg_attr(target_arch="wasm32", wasm_bindgen(start))]
pub async fn run() {
    // std::env::set_var("RUST_BACKTRACE", "1");

    
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            // web_sys::console::log_2(&"hello, world!".into());
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Info).expect("Couldn't initialize logger");

            log::info!("logging works!");

        } else {
            // env_logger::init();
        }
    }

    shaderunner::window::run_program::<GameProgram<ExampleGame>>().await;
}

pub fn main() {
    // std::env::set_var("RUST_BACKTRACE", "1");

    pollster::block_on(
        shaderunner::window::run_program::<GameProgram<ExampleGame>>()
    );
}

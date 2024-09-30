use std::time::Duration;

use cgmath::Vector3;
use crate::{renderer::Renderer, scene::{camera::Camera, light::{AmbientLight, DirectionalLight}, Transform}, ui_manager::UIManager, window::{game_loop::GameLoop, EventHandler, Game, Program}};
use winit::{event::{Event, WindowEvent}, event_loop::EventLoopWindowTarget, window::Window};


pub struct EchoesProgram<T: Game> {
    event_manager: EventHandler,
    game_loop: GameLoop,
    game: T,
}

impl<T: Game> Program for EchoesProgram<T> {
    fn new(window: Window) -> Self {
        EchoesProgram {
            event_manager: EventHandler::new(),
            game_loop: GameLoop::default(),
            game: pollster::block_on(T::new(window)),
        }
    }

    fn handle_event(
        &mut self,
        event: Event<()>,
        elwt: &EventLoopWindowTarget<()>,
    ) {
        self.event_manager.handle_event(
            &mut self.game_loop,
            &mut self.game,
            event,
            elwt,
        );
    }
}

pub struct RenderGame {
    renderer: Renderer,
    ui_manager: UIManager,
    window: Window,
}

impl RenderGame {
}

impl Game for RenderGame {
    async fn new(window: Window) -> RenderGame {
        let mut renderer = Renderer::new(&window).await;
    
        let cube_mesh = crate::test_assets::cube_mesh();
        let cube_mesh = renderer.add_mesh(&cube_mesh);
        let _instance0 = renderer.add_instance(cube_mesh, Transform::from_translation(Vector3::new(-0.5, -0.5, -0.5)));
    
        let sphere_mesh = crate::test_assets::simple_sphere_mesh(1.0, 16, Vector3::new(1.0, 1.0, 1.0));
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
        
        let ui_manager = UIManager::new(&window);
    
        RenderGame {
            renderer,
            ui_manager,
            window,
        }
    
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.renderer.resize(&new_size);
    }

    fn input(&mut self, _event: &WindowEvent) -> bool {
        false
    }

    fn update(&mut self) {

    }

    fn render(&mut self, _since_render: Duration, _since_update: Duration) {
        // self.renderer.render();
        let ui_frame = self.ui_manager.run(&self.window);
        self.renderer.render_with_ui(Some(&ui_frame));
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

    crate::window::run_program::<EchoesProgram<RenderGame>>().await;
}

pub mod game_loop;
pub mod timing_stats;

use std::time::Duration;

use winit::{
    application::ApplicationHandler, dpi::PhysicalSize, event::*, event_loop::{ActiveEventLoop, EventLoop}, window::{Window, WindowAttributes, WindowId}
};

use self::game_loop::GameLoop;

pub trait Program {
    fn new(window: Window) -> Self;
    fn handle_event(
        &mut self,
        event: Event<()>,
        event_loop: &ActiveEventLoop,
    );

}

struct OuterProgram<T: Program> {
    program: Option<T>,
}

impl<T: Program> OuterProgram<T> {
    pub fn new() -> Self {
        let program = None;
        
        Self {
            program
        }
    }
}

impl<T: Program> ApplicationHandler for OuterProgram<T> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let None = self.program { // can't use == because window doesn't implement PartialEq
            let size = PhysicalSize { width: 640, height: 512 };
            
            let window_attributes = WindowAttributes::default()
                .with_title("shaderunner app")
                .with_inner_size(size);
            let window = event_loop.create_window(window_attributes).unwrap();

            self.program = Some(T::new(window));
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if let Some(program) = self.program.as_mut() {
            program.handle_event(Event::WindowEvent { window_id, event }, event_loop);
        }
    }

    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        device_id: DeviceId,
        event: DeviceEvent,
    ) {
        if let Some(program) = self.program.as_mut() {
            program.handle_event(Event::DeviceEvent { device_id, event }, event_loop);
        }
    }

}

pub async fn run_program<T: Program + 'static>() {

    let event_loop = EventLoop::new().unwrap();

    // event_loop.set_control_flow(ControlFlow::Poll);

    #[cfg(target_arch = "wasm32")]
    {
        // window.set_inner_size(PhysicalSize::new(800, 600));
        
        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
        .and_then(|win| win.document())
        .and_then(|doc| {
            let dst = doc.get_element_by_id("wasm-example")?;
            let canvas = web_sys::Element::from(window.canvas()?);
            dst.append_child(&canvas).ok()?;
            Some(())
        })
        .expect("couldn't append canvas to document body");

        let size_result = window.request_inner_size(size);
        log::info!("size_result: {:?}", size_result);
    }

    let mut outer_program = OuterProgram::<T>::new();

    event_loop.run_app(&mut outer_program);

}



pub trait Game {
    #[allow(async_fn_in_trait)]
    /// Create a new game instance.  Do not allow `window` to drop; this will cause a swap chain changed error.
    async fn new(window: Window) -> Self;
    fn resize(&mut self, new_size: &winit::dpi::PhysicalSize<u32>);
    fn input(&mut self, event: &Event<()>) -> bool;
    fn update(&mut self);
    fn render(&mut self, since_render: Duration, since_update: Duration);
}

pub struct EventHandler {
}

impl EventHandler {
    pub fn new() -> EventHandler {
        EventHandler {
        }
    }

    pub fn handle_event<T: Game>(
        &mut self,
        game_loop: &mut GameLoop,
        game: &mut T,
        event: Event<()>,
        event_loop: &ActiveEventLoop,
    ) {
        let intercepted = match event {
            Event::WindowEvent {
                event: ref window_event,
                ..
            } =>  {
                self.handle_window_event(
                    game_loop,
                    game,
                    window_event,
                    event_loop,
                )
            },
            // Event::RedrawRequested(_) => {
            //   game.render(game_loop.since_render(), game_loop.since_update());  
            // },
            // Event::MainEventsCleared => {
            //     *control_flow = game_loop.update_or_render(game);
            // },
            Event::AboutToWait => {
                let control_flow = game_loop.update_or_render(game);
                event_loop.set_control_flow(control_flow);
                true
            }
            _ => false
        };

        if !intercepted {
            game.input(&event);
        }
    }

    fn handle_window_event<T: Game>(
        &mut self,
        game_loop: &GameLoop,
        game: &mut T,
        event: &WindowEvent,
        event_loop: &ActiveEventLoop,
    ) -> bool {
            match event {
                WindowEvent::CloseRequested => {
                    event_loop.exit();
                    true
                },
                WindowEvent::Resized(physical_size) => {
                    game.resize(physical_size);
                    true
                }
                WindowEvent::RedrawRequested => {
                    game.render(game_loop.since_render(), game_loop.since_update());
                    true
                }
                _ => false
            }
    }
}


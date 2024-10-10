pub mod game_loop;
pub mod timing_stats;

use std::time::Duration;

use winit::{
    dpi::PhysicalSize, event::*, event_loop::{EventLoop, EventLoopWindowTarget}, window::Window
};

use self::game_loop::GameLoop;

pub trait Program {
    fn new(window: Window) -> Self;
    fn handle_event(
        &mut self,
        event: Event<()>,
        elwt: &EventLoopWindowTarget<()>,
    );

}

pub async fn run_program<T: Program + 'static>() {

    let size = PhysicalSize { width: 640, height: 512 };

    let event_loop = EventLoop::new().unwrap();
    let window = winit::window::WindowBuilder::new()
        .with_title("shaderunner app")
        .with_inner_size(size)
        .build(&event_loop).unwrap();

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

    let mut program = T::new(window);

    event_loop.run(move |event, elwt| {
        program.handle_event(
            event,
            elwt,
        );
    }).expect("event loop run error");

}



pub trait Game {
    #[allow(async_fn_in_trait)]
    /// Create a new game instance.  Do not allow `window` to drop; this will cause a swap chain changed error.
    async fn new(window: Window) -> Self;
    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>);
    fn input(&mut self, event: &WindowEvent) -> bool;
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
        elwt: &EventLoopWindowTarget<()>,
    ) {
        match event {
            Event::WindowEvent {
                event: window_event,
                ..
            } =>  {
                self.handle_window_event(
                    game_loop,
                    game,
                    window_event,
                    elwt,
                );
            },
            // Event::RedrawRequested(_) => {
            //   game.render(game_loop.since_render(), game_loop.since_update());  
            // },
            // Event::MainEventsCleared => {
            //     *control_flow = game_loop.update_or_render(game);
            // },
            Event::AboutToWait => {
                let control_flow = game_loop.update_or_render(game);
                elwt.set_control_flow(control_flow);
            }
            _ => ()
        }
    }

    fn handle_window_event<T: Game>(
        &mut self,
        game_loop: &GameLoop,
        game: &mut T,
        event: WindowEvent,
        elwt: &EventLoopWindowTarget<()>,
    ) {
    
        if !game.input(&event) {
            match event {
                WindowEvent::CloseRequested => {
                    elwt.exit();
                },
                WindowEvent::Resized(physical_size) => {
                    game.resize(physical_size);
                }
                WindowEvent::RedrawRequested => {
                    game.render(game_loop.since_render(), game_loop.since_update());
                }
                _ => ()
            }
        }
    }
}


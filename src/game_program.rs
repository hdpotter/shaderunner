use winit::{event::{DeviceEvent, DeviceId, StartCause, WindowEvent}, event_loop::ActiveEventLoop, window::{Window, WindowId}};

use crate::window::{game_loop::GameLoop, Program};

use std::time::Duration;

pub trait Game {
    #[allow(async_fn_in_trait)]
    /// Create a new game instance.  Do not allow `window` to drop; this will cause a swap chain changed error.
    async fn new(window: Window) -> Self;
    fn resize(&mut self, new_size: &winit::dpi::PhysicalSize<u32>);
    fn update(&mut self);
    fn render(&mut self, since_render: Duration, since_update: Duration);

    fn window_event(&mut self, event: &WindowEvent) -> bool { let _ = event; false }
    fn device_event(&mut self, event: &DeviceEvent) -> bool { let _ = event; false }
}

pub struct GameProgram<T: Game> {
    game_loop: GameLoop,
    game: T,
}

impl<T: Game> Program for GameProgram<T> {
    fn new(window: Window) -> Self {
        GameProgram {
            game_loop: GameLoop::default(),
            game: pollster::block_on(T::new(window)),
        }
    }

    fn new_events(&mut self, _cause: &StartCause, event_loop: &ActiveEventLoop) {
        let control_flow = self.game_loop.update_or_render(&mut self.game);
        event_loop.set_control_flow(control_flow);
    }

    fn window_event(&mut self, event: &WindowEvent, _window_id: &WindowId, event_loop: &ActiveEventLoop) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(physical_size) => {
                self.game.resize(physical_size);
            }
            WindowEvent::RedrawRequested => {
                self.game.render(self.game_loop.since_render(), self.game_loop.since_update());
            }
            event => {
                self.game.window_event(event);
            }
        }
    }

    fn device_event(&mut self, event: &DeviceEvent, _device_id: &DeviceId, _event_loop: &ActiveEventLoop) {
        self.game.device_event(event);
    }
}
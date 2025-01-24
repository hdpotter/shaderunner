use winit::{event::Event, event_loop::ActiveEventLoop, window::Window};

use crate::window::{game_loop::GameLoop, EventHandler, Game, Program};

pub struct GameProgram<T: Game> {
    event_manager: EventHandler,
    game_loop: GameLoop,
    game: T,
}

impl<T: Game> Program for GameProgram<T> {
    fn new(window: Window) -> Self {
        GameProgram {
            event_manager: EventHandler::new(),
            game_loop: GameLoop::default(),
            game: pollster::block_on(T::new(window)),
        }
    }

    fn handle_event(
        &mut self,
        event: Event<()>,
        event_loop: &ActiveEventLoop,
    ) {
        self.event_manager.handle_event(
            &mut self.game_loop,
            &mut self.game,
            event,
            event_loop,
        );
    }
}
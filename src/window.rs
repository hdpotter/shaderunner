pub mod game_loop;
pub mod timing_stats;

use winit::{
    application::ApplicationHandler, dpi::PhysicalSize, event::*, event_loop::{ActiveEventLoop, EventLoop}, window::{Window, WindowAttributes, WindowId}
};

pub trait Program {
    fn new(window: Window) -> Self;
    
    fn window_event(
        &mut self,
        event: &WindowEvent,
        window_id: &WindowId,
        event_loop: &ActiveEventLoop,
    ) {
        let _ = (
            event,
            window_id,
            event_loop,
        );
    }

    fn device_event(
        &mut self,
        event: &DeviceEvent,
        device_id: &DeviceId,
        event_loop: &ActiveEventLoop,
    ) {
        let _ = (
            event,
            device_id,
            event_loop,
        );
    }
    
    fn new_events(
        &mut self,
        cause: &StartCause,
        event_loop: &ActiveEventLoop,
    ) {
        let _ = (
            cause,
            event_loop,
        );
    }

    fn about_to_wait(
        &mut self,
        event_loop: &ActiveEventLoop
    ) {
        let _ = (
            event_loop,
        );
    }

    fn resumed(
        &mut self,
        event_loop: &ActiveEventLoop,
    ) {
        let _ = (
            event_loop,
        );
    }

    fn suspended(
        &mut self,
        event_loop: &ActiveEventLoop,
    ) {
        let _ = (
            event_loop,
        );
    }

    fn exiting(
        &mut self,
        event_loop: &ActiveEventLoop,
    ) {
        let _ = (
            event_loop,
        );
    }

    fn memory_warning(
        &mut self,
        event_loop: &ActiveEventLoop,
    ) {
        let _ = (
            event_loop,
        );
    }

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
        match &mut self.program {
            None => {
                let size = PhysicalSize { width: 640, height: 512 };
                
                let window_attributes = WindowAttributes::default()
                    .with_title("shaderunner app")
                    .with_inner_size(size);
                let window = event_loop.create_window(window_attributes).unwrap();
        
                self.program = Some(T::new(window));
            },
            Some(program) => {
                program.resumed(event_loop);
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if let Some(program) = self.program.as_mut() {
            program.window_event(&event, &window_id, event_loop);
        }
    }

    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        device_id: DeviceId,
        event: DeviceEvent,
    ) {
        if let Some(program) = self.program.as_mut() {
            program.device_event(&event, &device_id, event_loop);
        }
    }

    fn new_events(
        &mut self,
        event_loop: &ActiveEventLoop,
        cause: StartCause,
    ) {
        if let Some(program) = self.program.as_mut() {
            program.new_events(&cause, event_loop);
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(program) = self.program.as_mut() {
            program.about_to_wait(event_loop);
        }
    }

    fn suspended(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(program) = self.program.as_mut() {
            program.suspended(event_loop);
        }
    }

    fn exiting(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(program) = self.program.as_mut() {
            program.exiting(event_loop);
        }
    }

    fn memory_warning(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(program) = self.program.as_mut() {
            program.memory_warning(event_loop);
        }
    }

}

pub async fn run_program<T: Program + 'static>() {

    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

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

    event_loop.run_app(&mut outer_program).unwrap();

}




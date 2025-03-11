use std::time::Duration;

use cgmath::Vector2;
use image::ImageBuffer;
use shaderunner::{handle::Handle, renderer::resources::texture::Texture, Game, GameProgram, Renderer};
use winit::{event::WindowEvent, window::Window};

pub struct TextureGame {
    renderer: Renderer,
    texture: Handle<Texture>,
}

impl Game for TextureGame {
    async fn new(window: Window) -> Self {
        let mut renderer = Renderer::new(window).await;

        let mut image = ImageBuffer::new(200, 200);
        for (x, y, pixel) in image.enumerate_pixels_mut() {
            if x >= 100 && y >= 100 {
                *pixel = image::Rgba([0, 0, 255, 255]);
            } else {
                *pixel = image::Rgba([255, 0, 0, 255]);
            }
        }

        let texture = renderer.add_texture(&image);

        Self {
            renderer,
            texture,
        }
    }

    fn resize(&mut self, new_size: &winit::dpi::PhysicalSize<u32>) {
        self.renderer.resize(&new_size);
    }

    fn window_event(&mut self, event: &WindowEvent) -> bool {
        self.renderer.egui_event(event)
    }

    fn update(&mut self) {

    }
    
    fn render(&mut self, _since_render: Duration, _since_update: Duration) {
        self.renderer.draw_texture(
            self.texture,
            Vector2::new(-0.5, -0.5),
            Vector2::new(0.5, 0.5),
        );

        // render
        self.renderer.render();
    }
}

fn main() {
    pollster::block_on(
        shaderunner::window::run_program::<GameProgram<TextureGame>>()
    )
}
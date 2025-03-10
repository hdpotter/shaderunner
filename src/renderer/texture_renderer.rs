use cgmath::Vector2;

use crate::{handle::Handle, sprite_vertex::SpriteVertex};

use super::{resizable_buffer::ResizableBuffer, resources::texture::Texture};


pub struct TextureRenderer {
    textures: Vec<Handle<Texture>>,
    vertices: Vec<SpriteVertex>,
    buffer: ResizableBuffer,
}

impl TextureRenderer {
    pub fn new(device: &wgpu::Device) -> Self {
        let textures = Vec::new();
        let vertices = Vec::new();

        let buffer = ResizableBuffer::new(
            32,
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            device
        );

        // let pipeline = {
        //     let shader = wgpu::ShaderModuleDescriptor {
        //         label: Some("line_shader"),
        //         source: wgpu::ShaderSource::Wgsl(include_str!("../line_shader.wgsl").into()),
        //     };
        //     let shader = device.create_shader_module(shader);

        //     let primitive = wgpu::PrimitiveState {
        //         topology: wgpu::PrimitiveTopology::TriangleList,
        //         front_face: wgpu::FrontFace::Ccw,
        //         .. Default::default()
        //     };

        //     let vertex_layout = SpriteVertex::vertex_buffer_layout();

        //     create_pipeline::create_render_pipeline(
        //         device,
        //         &pipeline_layout,
        //         color_format,
        //         Some(depth_format),
        //         &[vertex_layout],
        //         &shader,
        //         primitive,
        //     )
        // };

        Self {
            textures,
            vertices,
            buffer,
        }
    }

    pub fn draw_texture(
        &mut self,
        texture: Handle<Texture>,
        lower_left: Vector2<f32>,
        upper_right: Vector2<f32>
    ) {
        self.textures.push(texture);

        let min_x = lower_left.x;
        let max_x = upper_right.x;
        let min_y = lower_left.y;
        let max_y = upper_right.y;

        let lower_left = SpriteVertex::new(lower_left, Vector2::new(0.0, 0.0));
        let upper_right = SpriteVertex::new(upper_right, Vector2::new(1.0, 1.0));

        let upper_left = SpriteVertex::new(Vector2::new(min_x, max_y), Vector2::new(0.0, 1.0));
        let lower_right = SpriteVertex::new(Vector2::new(max_x, min_y), Vector2::new(1.0, 0.0));

        self.vertices.push(lower_left);
        self.vertices.push(upper_right);
        self.vertices.push(upper_left);

        self.vertices.push(lower_left);
        self.vertices.push(lower_right);
        self.vertices.push(upper_right);
    }

    pub fn update_buffer(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        if self.textures.len() > 0 {
            self.buffer.update(
                device,
                queue,
                bytemuck::cast_slice(&self.vertices),
            );
        }
    }

    pub fn render_and_clear(
        &mut self,
        render_pass: &mut wgpu::RenderPass,
    ) {
        if self.textures.len() > 0 {
            render_pass.set_vertex_buffer(0, self.buffer.buffer().slice(0..self.buffer.size() as u64));
            for i in 0..self.textures.len() {
                let start = 6 * i as u32;
                let end = 6 * (i + 1) as u32;

                render_pass.draw(start..end, 0..1);
            }

            self.textures.clear();
            self.vertices.clear();
        }

    }
}
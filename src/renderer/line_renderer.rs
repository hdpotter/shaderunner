use crate::color_vertex::ColorVertex;

use super::resizable_buffer::ResizableBuffer;

pub struct LineRenderer {
    vertices: Vec<ColorVertex>,
    buffer: ResizableBuffer,
    count: u32,
}

impl LineRenderer {
    pub fn new(device: &wgpu::Device) -> Self {
        let vertices = Vec::new();
        
        let buffer = ResizableBuffer::new(
            32,
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            device,
        );

        let count = vertices.len() as u32;

        Self {
            vertices,
            buffer,
            count,
        }
    }

    pub fn draw_line(&mut self, start: ColorVertex, end: ColorVertex) {
        self.vertices.push(start);
        self.vertices.push(end);
    }

    pub fn update_buffer_and_clear(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        if self.vertices.len() > 0 {
            self.buffer.update(
                device,
                queue,
                bytemuck::cast_slice(&self.vertices),
            );
            self.count = self.vertices.len() as u32;
            self.vertices.clear();
        }

    }

    pub fn render(
        &self,
        render_pass: &mut wgpu::RenderPass,
        camera_bind_group: &wgpu::BindGroup,
    ) {
        if self.buffer.size() > 0 {
            render_pass.set_vertex_buffer(0, self.buffer.buffer().slice(0..self.buffer.size() as u64));
            render_pass.set_bind_group(0, camera_bind_group, &[]);
            render_pass.draw(0..self.count, 0..1);
        }
    }
}
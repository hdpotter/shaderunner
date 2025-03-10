use crate::{color_vertex::ColorVertex, Vertex};

use super::{create_pipeline, resizable_buffer::ResizableBuffer};

pub struct LineRenderer {
    vertices: Vec<ColorVertex>,
    buffer: ResizableBuffer,
    pipeline: wgpu::RenderPipeline,
}

impl LineRenderer {
    pub fn new(
        device: &wgpu::Device,
        pipeline_layout: &wgpu::PipelineLayout,
        color_format: wgpu::TextureFormat,
        depth_format: wgpu::TextureFormat,
    ) -> Self {
        let vertices = Vec::new();
        
        let buffer = ResizableBuffer::new(
            32,
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            device,
        );

        let pipeline = {
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("line_shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("../line_shader.wgsl").into()),
            };
            let shader = device.create_shader_module(shader);

            let primitive = wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
                front_face: wgpu::FrontFace::Ccw,
                .. Default::default()
            };

            let vertex_layout = ColorVertex::vertex_buffer_layout();

            create_pipeline::create_render_pipeline(
                device,
                &pipeline_layout,
                color_format,
                Some(depth_format),
                &[vertex_layout],
                &shader,
                primitive,
            )
        };

        Self {
            vertices,
            buffer,
            pipeline,
        }
    }

    pub fn draw_line(&mut self, start: ColorVertex, end: ColorVertex) {
        self.vertices.push(start);
        self.vertices.push(end);
    }

    pub fn update_buffer(
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
        }
        
    }
    
    pub fn render_and_clear(
        &mut self,
        render_pass: &mut wgpu::RenderPass,
        camera_bind_group: &wgpu::BindGroup,
    ) {
        let count = self.vertices.len() as u32;

        if self.vertices.len() > 0 {
            render_pass.set_pipeline(&self.pipeline);

            render_pass.set_vertex_buffer(0, self.buffer.buffer().slice(0..self.buffer.size() as u64));
            render_pass.set_bind_group(0, camera_bind_group, &[]);
            render_pass.draw(0..count, 0..1);
        
            self.vertices.clear();
        }
    }
}
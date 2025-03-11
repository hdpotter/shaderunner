use std::collections::HashMap;

use cgmath::Vector2;

use crate::{handle::Handle, sprite_vertex::SpriteVertex};

use super::{create_pipeline, resizable_buffer::ResizableBuffer, resources::{texture::Texture, Resources}};


pub struct TextureRenderer {
    textures: Vec<Handle<Texture>>,
    vertices: Vec<SpriteVertex>,
    buffer: ResizableBuffer,
    pipeline: wgpu::RenderPipeline,
    texture_bind_groups: HashMap<Handle<Texture>, wgpu::BindGroup>,
}

impl TextureRenderer {
    pub fn new(
        device: &wgpu::Device,
        texture_bind_group_layout: &wgpu::BindGroupLayout,
        color_format: wgpu::TextureFormat,
        depth_format: wgpu::TextureFormat,
    ) -> Self {
        let textures = Vec::new();
        let vertices = Vec::new();

        let buffer = ResizableBuffer::new(
            32,
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            device
        );

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("TextureRenderer pipeline layout"),
            bind_group_layouts: &[
                texture_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let pipeline = {
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("sprite_shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("../sprite_shader.wgsl").into()),
            };
            let shader = device.create_shader_module(shader);

            let primitive = wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                front_face: wgpu::FrontFace::Ccw,
                .. Default::default()
            };

            let vertex_layout = SpriteVertex::vertex_buffer_layout();

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

        let texture_bind_groups = HashMap::new();

        Self {
            textures,
            vertices,
            buffer,
            pipeline,
            texture_bind_groups,
        }
    }

    pub fn ensure_bind_group_added(
        &mut self,
        texture: Handle<Texture>,
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        resources: &Resources,
    ) {
        match self.texture_bind_groups.get(&texture) {
            Some(_) => (),
            None => {
                let texture_resource = resources.texture(texture);

                let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(texture_resource.view()),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(texture_resource.sampler()),
                        }
                    ],
                    label: Some("texture bind group"),
                });

                self.texture_bind_groups.insert(texture, bind_group);
            }
        }
    }

    pub fn draw_texture(
        &mut self,
        texture: Handle<Texture>,
        lower_left: Vector2<f32>,
        upper_right: Vector2<f32>,
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        resources: &Resources,
    ) {
        self.textures.push(texture);
        self.ensure_bind_group_added(texture, device, layout, resources);

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
            
            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_vertex_buffer(0, self.buffer.buffer().slice(0..self.buffer.size() as u64));
            for i in 0..self.textures.len() {
                let texture = self.textures[i];

                render_pass.set_bind_group(0, &self.texture_bind_groups[&texture], &[]);

                let start = 6 * i as u32;
                let end = 6 * (i + 1) as u32;

                render_pass.draw(start..end, 0..1);
            }

            self.textures.clear();
            self.vertices.clear();
        }

    }
}
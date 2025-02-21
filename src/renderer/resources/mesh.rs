use wgpu::util::DeviceExt;

use crate::{MeshBuilder, Vertex};

/// A mesh on the GPU.
pub struct Mesh {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,
}

impl Mesh {
    pub fn index_count(&self) -> u32 {
        self.index_count
    }

    pub fn vertex_buffer(&self) -> &wgpu::Buffer {
        &self.vertex_buffer
    }

    pub fn index_buffer(&self) -> &wgpu::Buffer {
        &self.index_buffer
    }

    pub fn new_from_mesh_builder<T: Vertex>(mesh_builder: &MeshBuilder<T>, device: &wgpu::Device) -> Self {
        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("vertex buffer"),
                contents: bytemuck::cast_slice(mesh_builder.vertices()),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );
        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("index buffer"),
                contents: bytemuck::cast_slice(mesh_builder.indices()),
                usage: wgpu::BufferUsages::INDEX,
            }
        );
        let index_count = mesh_builder.indices().len() as u32;

        Self {
            vertex_buffer,
            index_buffer,
            index_count,
        }
    }
}
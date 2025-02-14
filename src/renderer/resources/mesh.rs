use wgpu::util::DeviceExt;

use crate::{MeshBuilder, Vertex};

/// A mesh on the GPU.
pub struct Mesh {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
}

impl Mesh {
    pub fn vertex_buffer(&self) -> &wgpu::Buffer {
        &self.vertex_buffer
    }

    pub fn index_buffer(&self) -> &wgpu::Buffer {
        &self.index_buffer
    }

    pub fn new(vertex_buffer: wgpu::Buffer, index_buffer: wgpu::Buffer) -> Self {
        Mesh {
            vertex_buffer,
            index_buffer,
        }
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

        Self::new(
            vertex_buffer,
            index_buffer,
        )
    }
}
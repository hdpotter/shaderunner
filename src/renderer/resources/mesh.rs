use wgpu::util::DeviceExt;

use crate::{MeshBuilder, Vertex};


/// A mesh on the GPU.
pub enum Mesh {
    Nonempty(InnerMesh),
    Empty,
}

impl Mesh {
    pub fn new_from_mesh_builder<T: Vertex>(mesh_builder: &MeshBuilder<T>, device: &wgpu::Device) -> Self {
        if mesh_builder.index_count() > 0 {
            let mesh = InnerMesh::new_from_mesh_builder(mesh_builder, device);

            Self::Nonempty(mesh)
        } else {
            Self::Empty
        }

    }
}



/// A mesh on the GPU that is guaranteed to be nonempty.
pub struct InnerMesh {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,
}

impl InnerMesh {
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
        let index_count = mesh_builder.indices().len() as u32;

        assert!(index_count > 0);

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

        Self {
            vertex_buffer,
            index_buffer,
            index_count,
        }
    }
}
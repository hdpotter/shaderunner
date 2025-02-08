use std::marker::PhantomData;

use cgmath::Vector3;
use wgpu::util::DeviceExt;


// todo: investigate whether static lifetime is appropriate
pub trait Vertex: Copy + Clone + bytemuck::Pod + bytemuck::Zeroable{
    fn position(&self) -> cgmath::Vector3<f32>;
    fn vertex_buffer_layout() -> wgpu::VertexBufferLayout<'static>;
}

#[derive(Debug, Copy, Clone)]
pub struct VertexReference<T: Vertex> {
    index: u32,
    phantom: PhantomData<T>,
}

impl<T: Vertex> VertexReference<T> {
    pub fn new(index: u32) -> VertexReference<T> {
        VertexReference {
            index,
            phantom: PhantomData,
        }
    }

    pub fn index(&self) -> u32 {
        self.index
    }
}

pub struct Mesh<T: Vertex> {
    vertices: Vec<T>,
    indices: Vec<u32>,
}

impl<T: Vertex> Mesh<T> {
    pub fn vertices(&self) -> &Vec<T> {
        &self.vertices
    }

    pub fn indices(&self) -> &Vec<u32> {
        &self.indices
    }

    pub fn new() -> Self {
        Self {
            vertices: vec![],
            indices: vec![],
        }
    }

    pub fn get_position(&self, vertex_reference: VertexReference<T>) -> Option<Vector3<f32>>{
        Some(self.vertices.get(vertex_reference.index() as usize)?.position())
    }

    pub fn add_vertex(&mut self, vertex: T) -> VertexReference<T> {
        self.vertices.push(vertex);
        VertexReference::new((self.vertices.len() - 1) as u32)
    }

    pub fn add_triangle_refs(&mut self, a: VertexReference<T>, b: VertexReference<T>, c: VertexReference<T>) {
        self.indices.push(a.index());
        self.indices.push(b.index());
        self.indices.push(c.index());
    }

    pub fn add_triangle(&mut self, a: T, b: T, c: T) {
        let a = self.add_vertex(a);
        let b = self.add_vertex(b);
        let c = self.add_vertex(c);
        self.add_triangle_refs(a, b, c)
    }

    pub fn add_quad(&mut self, a: T, b: T, c: T, d: T) {
        let a = self.add_vertex(a);
        let b = self.add_vertex(b);
        let c = self.add_vertex(c);
        let d = self.add_vertex(d);
        
        self.add_quad_refs(a, b, c, d);
    }

    pub fn add_quad_refs(&mut self, a: VertexReference<T>, b: VertexReference<T>, c: VertexReference<T>, d: VertexReference<T>) {
        self.add_triangle_refs(a, b, c);
        self.add_triangle_refs(a, c, d);
    }

    pub fn add_triangle_facing(&mut self, a: T, b: T, c: T, facing: cgmath::Vector3<f32>) {
        let normal = (a.position() - b.position()).cross(a.position() - c.position());
        if cgmath::dot(normal, facing) > 0.0 {
            self.add_triangle(a, b, c);
        } else {
            self.add_triangle(a, c, b);
        }
    }

    pub fn add_triangle_refs_facing(&mut self, a: VertexReference<T>, b: VertexReference<T>, c: VertexReference<T>, facing: cgmath::Vector3<f32>) {
        let normal = (self.get_position(a).unwrap() - self.get_position(b).unwrap()).cross(
            self.get_position(a).unwrap() - self.get_position(c).unwrap()
        );
        if cgmath::dot(normal, facing) > 0.0 {
            self.add_triangle_refs(a, b, c);
        } else {
            self.add_triangle_refs(a, c, b)
        }
    }

    pub fn add_quad_facing(&mut self, a: T, b: T, c: T, d: T, facing: cgmath::Vector3<f32>) {
        let normal = (a.position() - b.position()).cross(a.position() - c.position());
        if cgmath::dot(normal, facing) > 0.0 {
            self.add_quad(a, b, c, d);
        } else {
            self.add_quad(a, d, c, b);
        }
    }

    pub fn add_quad_refs_facing(&mut self, a: VertexReference<T>, b: VertexReference<T>, c: VertexReference<T>, d: VertexReference<T>, facing: cgmath::Vector3<f32>) {
        let normal = (self.get_position(a).unwrap() - self.get_position(b).unwrap()).cross(
            self.get_position(a).unwrap() - self.get_position(c).unwrap()
        );
        if cgmath::dot(normal, facing) > 0.0 {
            self.add_quad_refs(a, b, c, d);
        } else {
            self.add_quad_refs(a, d, c, b);
        }
    }

    pub fn export_vertex_buffer(&self, device: &wgpu::Device) -> wgpu::Buffer {
        device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("vertex buffer"),
                contents: bytemuck::cast_slice(&self.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            }
        )
    }

    pub fn export_index_buffer(&self, device: &wgpu::Device) -> wgpu::Buffer {
        device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("index buffer"),
                contents: bytemuck::cast_slice(&self.indices),
                usage: wgpu::BufferUsages::INDEX,
            }
        )
    }

    pub fn export_collider(&self) -> rapier3d::prelude::ColliderBuilder {
        use rapier3d::prelude::*;
        
        let mut vertices = Vec::new();
        for vertex in &self.vertices {
            let position = vertex.position();
            vertices.push(point![position.x, position.y, position.z]);
        }

        let mut indices = Vec::new();
        for i in (0..self.indices.len()).step_by(3) {
            indices.push([self.indices[i], self.indices[i+1], self.indices[i+2]]);
        }

        ColliderBuilder::trimesh(vertices, indices)
    }

    pub fn index_count(&self) -> u32 {
        self.indices.len() as u32
    }
}



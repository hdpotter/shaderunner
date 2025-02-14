use cgmath::Vector3;

use crate::mesh_builder::Vertex;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ColorVertex {
    position: [f32; 3],
    color: [f32; 3],
}

// todo: investigate performance implications of using into() everywhere
impl ColorVertex {
    pub fn new(position: Vector3<f32>, color: Vector3<f32>) -> ColorVertex {
        ColorVertex {
            position: position.into(),
            color: color.into(),
        }
    }

    pub fn new_white(position: Vector3<f32>) -> ColorVertex {
        Self::new(position, Vector3::new(1.0, 1.0, 1.0))
    }
}

impl Vertex for ColorVertex {
    fn position(&self) -> Vector3<f32> {
        self.position.into()
    }

    fn vertex_buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<ColorVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ]
        }
    }
}
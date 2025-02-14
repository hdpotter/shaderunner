use cgmath::Vector3;

use crate::mesh_builder::Vertex;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ColorNormalVertex {
    position: [f32; 3],
    color: [f32; 3],
    normal: [f32; 3],
}

// todo: investigate performance implications of using into() everywhere
impl ColorNormalVertex {
    pub fn new(position: Vector3<f32>, color: Vector3<f32>, normal: Vector3<f32>) -> ColorNormalVertex {
        ColorNormalVertex {
            position: position.into(),
            color: color.into(),
            normal: normal.into(),
        }
    }

    pub fn new_white(position: Vector3<f32>, normal: Vector3<f32>) -> ColorNormalVertex {
        Self::new(position, Vector3::new(1.0, 1.0, 1.0), normal)
    }
}

impl Vertex for ColorNormalVertex {
    fn position(&self) -> Vector3<f32> {
        self.position.into()
    }

    fn vertex_buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<ColorNormalVertex>() as wgpu::BufferAddress,
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
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                }
            ]
        }
    }
}
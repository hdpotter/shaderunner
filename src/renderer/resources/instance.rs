use crate::{handle::Handle, scene::Transform};

use super::instance_list::InstanceList;


pub struct Instance {
    transform: Transform,
    active: bool,
}

impl Instance {
    pub fn active(&self) -> bool {
        self.active
    }

    pub fn new(transform: Transform) -> Instance {
        Instance {
            transform,
            active: true,
        }
    }

    pub fn to_data(&self) -> InstanceData {
        let model = 
            cgmath::Matrix4::from_translation(self.transform.translation()) *
            cgmath::Matrix4::from(self.transform.rotation()) * 
            cgmath::Matrix4::from_scale(self.transform.scale());

        InstanceData {
            model: model.into(),
            rotation: cgmath::Matrix3::from(self.transform.rotation()).into(),
        }
    }

    pub fn set_transform(&mut self, transform: Transform) {
        self.transform = transform;
    }

    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }
}


#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceData {
    model: [[f32; 4]; 4],
    rotation: [[f32; 3]; 3],
}

impl InstanceData {
    pub const IDENTITY: InstanceData = InstanceData {
        model: [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 0.0]],
        rotation: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
    };

    pub fn vertex_buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceData>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
                    shader_location: 9,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 19]>() as wgpu::BufferAddress,
                    shader_location: 10,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 22]>() as wgpu::BufferAddress,
                    shader_location: 11,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ]
        }
    }
}

pub struct InstanceRef {
    list: Handle<InstanceList>,
    instance: Handle<Instance>,
}

impl InstanceRef {
    pub fn list(&self) -> Handle<InstanceList> {
        self.list
    }

    pub fn instance(&self) -> Handle<Instance> {
        self.instance
    }

    pub fn new(list: Handle<InstanceList>, instance: Handle<Instance>) -> Self {
        Self {
            list,
            instance,
        }
    }
}
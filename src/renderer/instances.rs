use generational_arena::{Index, Arena};
use crate::scene::Transform;
use crate::renderer::gpu_resources::MeshHandle;
use super::resizable_buffer::ResizableBuffer;


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

pub struct InstanceListResource {
    mesh: MeshHandle,
    instances: Arena<Instance>,
    instance_data: Vec<InstanceData>,
    instance_buffer: ResizableBuffer,
}

impl InstanceListResource {
    pub fn mesh(&self) -> MeshHandle {
        self.mesh
    }

    pub fn instance_buffer(&self) -> &wgpu::Buffer {
        &self.instance_buffer.buffer()
    }

    pub fn instance_count(&self) -> u32 {
        self.instances.len() as u32
    }

    pub fn buffered_instance_count(&self) -> u32 {
        self.instance_data.len() as u32
    }

    pub fn new(mesh: MeshHandle, device: &wgpu::Device) -> InstanceListResource {
        let instances = Arena::new();
        let instance_data = Vec::new();
        // let instance_buffer = device.create_buffer_init(
        //     &wgpu::util::BufferInitDescriptor {
        //         label: Some("instance buffer"),
        //         contents: bytemuck::cast_slice(&instance_data),
        //         usage: wgpu::BufferUsages::VERTEX,
        //     }
        // );

        let instance_buffer = ResizableBuffer::new(
            100,
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            device,
        );

        InstanceListResource {
            mesh,
            instances,
            instance_data,
            instance_buffer,
        }
    }

    pub fn add_instance(&mut self, transform: Transform) -> InstanceHandle {
        let instance = Instance::new(transform);
        let index = self.instances.insert(instance);
        InstanceHandle::new(self.mesh(), index)
    }

    pub fn update_instance(&mut self, instance: InstanceHandle, transform: Transform) {
        self.instances.get_mut(instance.index()).unwrap().set_transform(transform);
    }

    pub fn set_instance_active(&mut self, instance: InstanceHandle, active: bool) {
        self.instances.get_mut(instance.index()).unwrap().set_active(active);
    }

    pub fn remove_instance(&mut self, instance: InstanceHandle) {
        self.instances.remove(instance.index());
    }

    pub fn build_instance_buffer(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        // copy all instances into buffer
        self.instance_data.clear();
        for (_, instance) in &self.instances {
            if instance.active() {
                // todo: make separate list of active instances
                self.instance_data.push(instance.to_data());
            }
        }

        // upload buffer to gpu
        self.instance_buffer.update(
            device,
            &queue,
            bytemuck::cast_slice(&self.instance_data),
        );
    }
}

#[derive(Copy, Clone)]
pub struct InstanceHandle {
    mesh: MeshHandle,
    index: Index,
}

impl InstanceHandle {
    pub fn mesh(&self) -> MeshHandle {
        self.mesh
    }

    pub fn index(&self) -> Index {
        self.index
    }

    pub fn new(mesh: MeshHandle, index: Index) -> Self {
        InstanceHandle {
            mesh,
            index,
        }
    }
}
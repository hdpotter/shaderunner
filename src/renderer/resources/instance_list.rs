use generational_arena::Arena;

use crate::{handle::Handle, renderer::resizable_buffer::ResizableBuffer, Transform};

use super::{instance::{Instance, InstanceData}, mesh::Mesh, pipeline::Pipeline};

pub struct InstanceList {
    mesh: Handle<Mesh>,
    pipeline: Handle<Pipeline>,

    instances: Arena<Instance>,

    instance_data: Vec<InstanceData>,
    instance_buffer: ResizableBuffer,
}

impl InstanceList {
    pub fn pipeline(&self) -> Handle<Pipeline> {
        self.pipeline
    }

    pub fn mesh(&self) -> Handle<Mesh> {
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

    pub fn new(
        mesh: Handle<Mesh>,
        pipeline: Handle<Pipeline>,
        device: &wgpu::Device,
    ) -> Self {
        let instances = Arena::new();
        let instance_data = Vec::new();

        let instance_buffer = ResizableBuffer::new(
            100,
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            device,
        );

        Self {
            mesh,
            pipeline,
            instances,
            instance_data,
            instance_buffer,
        }
    }

    pub fn add_instance(&mut self, transform: Transform) -> Handle<Instance> {
        let instance = Instance::new(transform);
        Handle::insert(&mut self.instances, instance)
    }

    pub fn update_instance(&mut self, instance: Handle<Instance>, transform: Transform) {
        self.instances.get_mut(instance.index()).unwrap().set_transform(transform);
    }

    pub fn set_instance_active(&mut self, instance: Handle<Instance>, active: bool) {
        self.instances.get_mut(instance.index()).unwrap().set_active(active);
    }

    pub fn remove_instance(&mut self, instance: Handle<Instance>) {
        self.instances.remove(instance.index());
    }

    pub fn build_and_upload_instance_buffer(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
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
use cgmath::{EuclideanSpace, InnerSpace, Point3};

use crate::{AmbientLight, Camera, DirectionalLight};



pub struct CameraResource {
    camera_data: CameraData,
    camera_buffer: wgpu::Buffer,
}

impl CameraResource {
    pub fn camera_buffer(&self) -> &wgpu::Buffer {
        &self.camera_buffer
    }

    pub fn new(device: &wgpu::Device) -> Self {
        let camera_data = CameraData {
            position: [0.0, 0.0, 0.0, 0.0],
            view_proj: [
                [0.0, 0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0, 0.0]],
        };
        
        let camera_buffer = device.create_buffer(
            &wgpu::BufferDescriptor {
                label: Some("camera buffer"),
                size: std::mem::size_of::<CameraData>() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }
        );

        Self {
            camera_buffer,
            camera_data,
        }
    }

    pub fn update(&mut self, camera: &Camera, queue: &wgpu::Queue) {
        self.camera_data.position = Point3::from_vec(camera.eye()).to_homogeneous().into();
        self.camera_data.view_proj = camera.build_view_projection_matrix().into();
        queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.camera_data]));
    }
}


#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraData {
    position: [f32; 4],
    view_proj: [[f32; 4]; 4],
}

pub struct LightResource {
    light_data: LightData,
    light_buffer: wgpu::Buffer,
}

impl LightResource {
    pub fn light_buffer(&self) -> &wgpu::Buffer {
        &self.light_buffer
    }

    pub fn new(device: &wgpu::Device) -> LightResource {
        let light_data = LightData {
            direction: [0.0, 0.0, 0.0],
            color: [0.0, 0.0, 0.0],

            ambient_color: [0.0, 0.0, 0.0],

            _padding0: 0.0,
            _padding1: 0.0,
            _padding2: 0.0,
        };

        let light_buffer = device.create_buffer( &wgpu::BufferDescriptor {
            label: Some("directional light buffer"),
            size: std::mem::size_of::<LightData>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        LightResource {
            light_data,
            light_buffer,
        }
    }

    pub fn update(&mut self, directional_light: &DirectionalLight, ambient_light: &AmbientLight, queue: &wgpu::Queue) {
        self.light_data.direction = directional_light.direction().normalize().into();
        self.light_data.color = (directional_light.intensity() * directional_light.color()).into();

        self.light_data.ambient_color = (ambient_light.intensity() * ambient_light.color()).into();

        queue.write_buffer(
            &self.light_buffer,
            0,
            bytemuck::cast_slice(&[self.light_data]),
        );
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightData {
    direction: [f32; 3],
    _padding0: f32,
    color: [f32; 3],
    _padding1: f32,
    ambient_color: [f32; 3],
    _padding2: f32,
}
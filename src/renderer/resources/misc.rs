use crate::{renderer::texture::create_depth_texture, scene::camera::create_camera_bind_group_and_layout, AmbientLight, Camera, DirectionalLight};

use super::uniforms::{CameraResource, LightResource};


// we'll handle these in a much cleaner way after we finish the pipeline rewrite
pub struct Misc {
    camera: CameraResource,
    lights: LightResource,

    camera_bind_group_layout: wgpu::BindGroupLayout,
    camera_bind_group: wgpu::BindGroup,
    
    pipeline_layout: wgpu::PipelineLayout,

    depth_texture: wgpu::Texture,
    depth_texture_view: wgpu::TextureView,
}

impl Misc {
    pub fn pipeline_layout(&self) -> &wgpu::PipelineLayout {
        &self.pipeline_layout
    }

    pub fn camera_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.camera_bind_group_layout
    }

    pub fn camera_bind_group(&self) -> &wgpu::BindGroup {
        &self.camera_bind_group
    }

    pub fn depth_texture(&self) -> &wgpu::Texture {
        &self.depth_texture
    }

    pub fn depth_texture_view(&self) -> &wgpu::TextureView {
        &self.depth_texture_view
    }
    
    pub fn new(config: &wgpu::SurfaceConfiguration, device: &wgpu::Device) -> Self {
        let camera = CameraResource::new(device);
        let lights = LightResource::new(device);

        
        let (
            camera_bind_group_layout,
            camera_bind_group
        ) = create_camera_bind_group_and_layout(&camera.camera_buffer(), &lights.light_buffer(), device);
        
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pipeline layout"),
            bind_group_layouts: &[
                &camera_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let (
            depth_texture,
            depth_texture_view,
        ) = create_depth_texture(device, config);


        Self {
            camera,
            lights,
            camera_bind_group_layout,
            camera_bind_group,
            pipeline_layout,
            depth_texture,
            depth_texture_view,
        }
    }

    pub fn update_camera(&mut self, camera: &Camera, queue: &wgpu::Queue) {
        self.camera.update(camera, queue);
    }

    pub fn update_light(&mut self, directional_light: &DirectionalLight, ambient_light: &AmbientLight, queue: &wgpu::Queue) {
        self.lights.update(directional_light, ambient_light, queue);
    }

    pub fn resize_depth_texture(&mut self, device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) {
        (self.depth_texture, self.depth_texture_view) = create_depth_texture(device, config);
    }

}
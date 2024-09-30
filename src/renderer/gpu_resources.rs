use std::mem;

use cgmath::{Point3, EuclideanSpace, InnerSpace};
use generational_arena::{Arena, Index};

use crate::{mesh::Mesh, color_normal_vertex::ColorNormalVertex, scene::{Transform, camera::{Camera, create_camera_bind_group_and_layout}, light::{DirectionalLight, AmbientLight}}};

use super::{instances::{InstanceListResource, InstanceHandle}, texture::create_depth_texture};

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
                size: mem::size_of::<CameraData>() as u64,
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
            size: mem::size_of::<LightData>() as u64,
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

pub struct MeshResource {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,
}

impl MeshResource {
    pub fn vertex_buffer(&self) -> &wgpu::Buffer {
        &self.vertex_buffer
    }

    pub fn index_buffer(&self) -> &wgpu::Buffer {
        &self.index_buffer
    }

    pub fn index_count(&self) -> u32 {
        self.index_count
    }

    pub fn new(vertex_buffer: wgpu::Buffer, index_buffer: wgpu::Buffer, index_count: u32) -> Self {
        MeshResource {
            vertex_buffer,
            index_buffer,
            index_count,
        }
    }
}

#[derive(Copy, Clone)]
pub struct MeshHandle(Index);


// manages buffers, bind group layouts, and bind groups
pub struct Resources {
    meshes: Arena<MeshResource>,
    instances: Arena<InstanceListResource>,
    camera: CameraResource,
    light: LightResource,

    camera_bind_group_layout: wgpu::BindGroupLayout,
    camera_bind_group: wgpu::BindGroup,

    depth_texture: wgpu::Texture,
    depth_texture_view: wgpu::TextureView,
}

impl Resources {
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Resources {
        let meshes = Arena::new();
        let instances = Arena::new();
        let camera = CameraResource::new(device);
        let light = LightResource::new(device);

        // todo: consider refactoring bind groups into their own struct
        let (
            camera_bind_group_layout,
            camera_bind_group
        ) = create_camera_bind_group_and_layout(&camera.camera_buffer, &light.light_buffer, device);

        let (
            depth_texture,
            depth_texture_view,
        ) = create_depth_texture(device, config);

        Resources {
            meshes,
            instances,
            camera,
            light,

            camera_bind_group_layout,
            camera_bind_group,

            depth_texture,
            depth_texture_view,
        }
    }

    // ================================================================
    // bind groups and layouts
    // ================================================================
    pub fn camera_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.camera_bind_group_layout
    }

    pub fn camera_bind_group(&self) -> &wgpu::BindGroup {
        &self.camera_bind_group
    }

    // ================================================================
    // bind groups and layouts
    // ================================================================
    pub fn depth_texture(&self) -> &wgpu::Texture {
        &self.depth_texture
    }

    pub fn depth_texture_view(&self) -> &wgpu::TextureView {
        &self.depth_texture_view
    }

    pub fn resize_depth_texture(&mut self, device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) {
        (self.depth_texture, self.depth_texture_view) = create_depth_texture(device, config);
    }

    // ================================================================
    // meshes
    // ================================================================
    pub fn add_mesh(&mut self, mesh: &Mesh<ColorNormalVertex>, device: &wgpu::Device) -> MeshHandle {
        // set up mesh
        let vertex_buffer = mesh.export_vertex_buffer(device);
        let index_buffer = mesh.export_index_buffer(device);
        let index_count = mesh.index_count();
 
        let mesh_resource = MeshResource {
            vertex_buffer,
            index_buffer,
            index_count,
        };
        let index = self.meshes.insert(mesh_resource);
        let mesh_handle =  MeshHandle(index);

        // set up instance list
        let instance_list = InstanceListResource::new(mesh_handle, device);
        self.instances.insert(instance_list);

        // return handle
        mesh_handle
    }

    pub fn get_mesh(&self, handle: MeshHandle) -> &MeshResource {
        let MeshHandle(index) = handle;
        self.meshes.get(index).unwrap()
    }

    pub fn remove_mesh(&mut self, handle: MeshHandle) {
        let MeshHandle(index) = handle;
        self.meshes.remove(index);
    }

    // ================================================================
    // instances
    // ================================================================
    pub fn add_instance(&mut self, mesh: MeshHandle, transform: Transform) -> InstanceHandle {
        self.get_instance_list_mut(mesh).add_instance(transform)
    }

    pub fn update_instance(&mut self, instance: InstanceHandle, transform: Transform) {
        self.get_instance_list_mut(instance.mesh()).update_instance(instance, transform);
    }

    pub fn set_instance_active(&mut self, instance: InstanceHandle, active: bool) {
        self.get_instance_list_mut(instance.mesh()).set_instance_active(instance, active)
    }

    pub fn remove_instance(&mut self, instance: InstanceHandle) {
        self.get_instance_list_mut(instance.mesh()).remove_instance(instance);
    }

    pub fn iterate_instance_lists(&self) -> ArenaIterator<InstanceListResource> {
        ArenaIterator::iterate(&self.instances)
    }

    pub fn iterate_instance_lists_mut(&mut self) -> ArenaIteratorMut<InstanceListResource> {
        ArenaIteratorMut::iterate(&mut self.instances)
    }

    // ================================================================
    // camera and lights
    // ================================================================
    pub fn update_camera(&mut self, camera: &Camera, queue: &wgpu::Queue) {
        self.camera.update(camera, queue);
    }

    pub fn update_light(&mut self, directional_light: &DirectionalLight, ambient_light: &AmbientLight, queue: &wgpu::Queue) {
        self.light.update(directional_light, ambient_light, queue);
    }
    

    // ================================================================
    // utility
    // ================================================================
    fn get_instance_list_mut(&mut self, mesh: MeshHandle) -> &mut InstanceListResource {
        self.instances.get_mut(mesh.0).unwrap()
    }
}


pub struct ArenaIterator<'a, T> {
    iterator: generational_arena::Iter<'a, T>
}

impl<'a, T> ArenaIterator<'a, T> {
    pub fn iterate(arena: &'a Arena<T>) -> ArenaIterator<'a, T> {
        let iterator = arena.iter();
        ArenaIterator::<'a> {
            iterator,
        }
    }
}

impl<'a, T> Iterator for ArenaIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iterator.next() {
            Some((_, item)) => Some(item),
            None => None,
        }
    }
}

pub struct ArenaIteratorMut<'a, T> {
    iterator: generational_arena::IterMut<'a, T>
}

impl <'a, T> ArenaIteratorMut<'a, T> {
    pub fn iterate(arena: &'a mut Arena<T>) -> ArenaIteratorMut<'a, T> {
        let iterator = arena.iter_mut();
        ArenaIteratorMut {
            iterator,
        }
    }
}

impl<'a, T> Iterator for ArenaIteratorMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iterator.next() {
            Some((_, item)) => Some(item),
            None => None,
        }
    }
}


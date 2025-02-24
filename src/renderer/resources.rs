use std::collections::HashMap;

use arena_iterator::{ArenaIterator, ArenaIteratorMut};
use generational_arena::Arena;
use instance::InstanceRef;
use instance_list::InstanceList;
use mesh::Mesh;
use pipeline::Pipeline;
use uniforms::{CameraResource, LightResource};

use crate::{handle::Handle, scene::camera::create_camera_bind_group_and_layout, AmbientLight, Camera, DirectionalLight, MeshBuilder, Transform, Vertex};

use super::{create_pipeline::create_render_pipeline, texture::create_depth_texture};


pub mod pipeline;
pub mod mesh;
pub mod instance_list;
pub mod instance;
pub mod uniforms;
pub mod arena_iterator;


// Where should we put the dependents lists for meshes and pipelines?  The structure is simpler to
// understand if we do instance list -> mesh/pipeline links in instance list and back links as hashtables.
// We need to sort by pipeline when rendering, though, as shader switching is expensive, so in practice
// pipelines have a practical need to understand their dependents.

pub struct Resources {
    pipelines: Arena<Pipeline>,
    meshes: Arena<Mesh>,
    instance_lists: Arena<InstanceList>,

    pipeline_dependents: HashMap<Handle<Pipeline>, Arena<Handle<InstanceList>>>,
    mesh_dependents: HashMap<Handle<Mesh>, Arena<Handle<InstanceList>>>,

    // we'll handle these in a much cleaner way after we finish the pipeline rewrite
    camera: CameraResource,
    lights: LightResource,

    camera_bind_group_layout: wgpu::BindGroupLayout,
    camera_bind_group: wgpu::BindGroup,
    
    pipeline_layout: wgpu::PipelineLayout,

    depth_texture: wgpu::Texture,
    depth_texture_view: wgpu::TextureView,
}

impl Resources {
    pub fn mesh(&self, mesh: Handle<Mesh>) -> &Mesh {
        &self.meshes[mesh]
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
        let pipelines = Arena::new();
        let meshes = Arena::new();
        let instance_lists = Arena::new();

        let pipeline_dependents = HashMap::new();
        let mesh_dependents = HashMap::new();

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
            pipelines,
            meshes,
            instance_lists,
            pipeline_dependents,
            mesh_dependents,
            camera,
            lights,
            camera_bind_group_layout,
            camera_bind_group,
            pipeline_layout,
            depth_texture,
            depth_texture_view,
        }
    }

    pub fn add_pipeline(
        &mut self,
        color_format: wgpu::TextureFormat,
        depth_format: wgpu::TextureFormat,
        vertex_layouts: &[wgpu::VertexBufferLayout<'_>],
        shader: &wgpu::ShaderModule,
        primitive: wgpu::PrimitiveState,
        device: &wgpu::Device,
    ) -> Handle<Pipeline> {
        // create pipeline
        let pipeline = create_render_pipeline(
            device,
            &self.pipeline_layout,
            color_format,
            Some(depth_format),
            vertex_layouts,
            shader,
            primitive
        );
        let pipeline = Pipeline::new(pipeline);

        // add to arena
        let handle = Handle::insert(&mut self.pipelines, pipeline);

        // add dependent list
        let dependents = Arena::new();
        self.pipeline_dependents.insert(handle, dependents);

        // return
        handle
    }

    pub fn add_mesh<T: Vertex>(
        &mut self,
        mesh_builder: &MeshBuilder<T>,
        device: &wgpu::Device,
    ) -> Handle<Mesh> {
        // create and add mesh
        let mesh = Mesh::new_from_mesh_builder(mesh_builder, device);
        let handle = Handle::insert(&mut self.meshes, mesh);

        // add dependent list
        let dependents = Arena::new();
        self.mesh_dependents.insert(handle, dependents);

        // return
        handle
    }

    pub fn add_instance_list(
        &mut self,
        pipeline: Handle<Pipeline>,
        mesh: Handle<Mesh>,
        device: &wgpu::Device,
    ) -> Handle<InstanceList> {
        // add instance list
        let instance_list = InstanceList::new(mesh, pipeline, device);
        let handle = Handle::insert(&mut self.instance_lists, instance_list);

        // add as dependent
        self.pipeline_dependents.get_mut(&pipeline).unwrap().insert(handle);
        self.mesh_dependents.get_mut(&mesh).unwrap().insert(handle);

        // return
        handle
    }

    pub fn remove_instance_list(
        &mut self,
        instance_list: Handle<InstanceList>
    ) {
        // get pipeline and mesh handles
        let instance_list_struct = &self.instance_lists[instance_list];
        let pipeline = instance_list_struct.pipeline();
        let mesh = instance_list_struct.mesh();

        // remove as dependent
        self.pipeline_dependents.get_mut(&pipeline).unwrap().remove(instance_list.index());
        self.mesh_dependents.get_mut(&mesh).unwrap().remove(instance_list.index());

        // remove list
        self.instance_lists.remove(instance_list.index());
    }

    pub fn remove_pipeline(
        &mut self,
        pipeline: Handle<Pipeline>
    ) {
        // verify no dependents and remove dependent list
        if self.pipeline_dependents[&pipeline].len() > 0 {
            panic!("attempted to remove pipeline with at least one dependent instance list");
        }
        self.pipeline_dependents.remove(&pipeline);

        // remove pipeline
        self.pipelines.remove(pipeline.index());
    }

    pub fn remove_mesh(
        &mut self,
        mesh: Handle<Mesh>
    ) {
        // verify no dependents and remove dependent list
        if self.mesh_dependents[&mesh].len() > 0 {
            panic!("attempted to remove mesh with at least one dependent instance list");
        }
        self.mesh_dependents.remove(&mesh);

        // remove pipeline
        self.pipelines.remove(mesh.index());
    }

    pub fn add_instance(&mut self, list: Handle<InstanceList>, transform: Transform) -> InstanceRef {
        let instance = self.instance_lists[list].add_instance(transform);
        InstanceRef::new(list, instance)
    }

    pub fn update_instance(&mut self, instance: InstanceRef, transform: Transform) {
        self.instance_lists[instance.list()].update_instance(instance.instance(), transform);
    }

    pub fn set_instance_active(&mut self, instance: InstanceRef, active: bool) {
        self.instance_lists[instance.list()].set_instance_active(instance.instance(), active);
    }

    pub fn remove_instance(&mut self, instance: InstanceRef) {
        self.instance_lists[instance.list()].remove_instance(instance.instance());
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

    pub fn iterate_instance_lists(&self) -> ArenaIterator<InstanceList> {
        ArenaIterator::iterate(&self.instance_lists)
    }

    pub fn iterate_instance_lists_mut(&mut self) -> ArenaIteratorMut<InstanceList> {
        ArenaIteratorMut::iterate(&mut self.instance_lists)
    }
}



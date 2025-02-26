use std::collections::HashMap;

use arena_iterator::{ArenaIterator, ArenaIteratorMut};
use generational_arena::Arena;
use instance::InstanceRef;
use instance_list::InstanceList;
use mesh::Mesh;
use pipeline::Pipeline;

use crate::{handle::Handle, MeshBuilder, Transform, Vertex};

use super::create_pipeline::create_render_pipeline;


pub mod pipeline;
pub mod mesh;
pub mod instance_list;
pub mod instance;
pub mod uniforms;
pub mod arena_iterator;
pub mod many_one;
pub mod misc;


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
    dependent_handles_pipeline_mesh: HashMap<Handle<InstanceList>, (Handle<Handle<InstanceList>>, Handle<Handle<InstanceList>>)>, //todo: better way?
}

impl Resources {
    pub fn mesh(&self, mesh: Handle<Mesh>) -> &Mesh {
        &self.meshes[mesh]
    }

    // pub fn pipelines(&self) -> &Arena<Pipeline> {
    //     &self.pipelines
    // }

    pub fn instance_lists(&self) -> &Arena<InstanceList> {
        &self.instance_lists
    }

    // // todo: more elegant way of handling this, and in general iteration over things to render
    // pub fn instance_list(&self, instance_list: Handle<InstanceList>) -> &InstanceList {
    //     &self.instance_lists[instance_list]
    // }

    pub fn new() -> Self {
        let pipelines = Arena::new();
        let meshes = Arena::new();
        let instance_lists = Arena::new();

        let pipeline_dependents = HashMap::new();
        let mesh_dependents = HashMap::new();
        let dependent_handles_pipeline_mesh = HashMap::new();

        Self {
            pipelines,
            meshes,
            instance_lists,
            pipeline_dependents,
            mesh_dependents,
            dependent_handles_pipeline_mesh,
        }
    }

    pub fn add_pipeline(
        &mut self,
        color_format: wgpu::TextureFormat,
        depth_format: wgpu::TextureFormat,
        vertex_layouts: &[wgpu::VertexBufferLayout<'_>],
        pipeline_layout: &wgpu::PipelineLayout,
        shader: &wgpu::ShaderModule,
        primitive: wgpu::PrimitiveState,
        device: &wgpu::Device,
    ) -> Handle<Pipeline> {
        // create pipeline
        let pipeline = create_render_pipeline(
            device,
            pipeline_layout,
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
        let pipeline_handle = Handle::insert(
            &mut self.pipeline_dependents.get_mut(&pipeline).unwrap(),
            handle
        );
        let mesh_handle = Handle::insert(
            &mut self.mesh_dependents.get_mut(&mesh).unwrap(),
            handle
        );
        self.dependent_handles_pipeline_mesh.insert(
            handle,
            (pipeline_handle, mesh_handle)
        );

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
        let (pipeline_handle, mesh_handle) = self.dependent_handles_pipeline_mesh.remove(&instance_list).unwrap();
        self.pipeline_dependents.get_mut(&pipeline).unwrap().remove(pipeline_handle.index());
        self.mesh_dependents.get_mut(&mesh).unwrap().remove(mesh_handle.index());

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


    pub fn iterate_instance_lists(&self) -> ArenaIterator<InstanceList> {
        ArenaIterator::iterate(&self.instance_lists)
    }

    pub fn iterate_instance_lists_mut(&mut self) -> ArenaIteratorMut<InstanceList> {
        ArenaIteratorMut::iterate(&mut self.instance_lists)
    }

    pub fn iterate_pipelines(&self) -> ArenaIterator<Pipeline> {
        ArenaIterator::iterate(&self.pipelines)
    }

    pub fn iterate_pipeline_dependents(&self, pipeline: Handle<Pipeline>) -> ArenaIterator<Handle<InstanceList>> {
        let dependents = &self.pipeline_dependents[&pipeline];
        ArenaIterator::iterate(dependents)
    }

}



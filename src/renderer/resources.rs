use std::collections::HashMap;

use generational_arena::Arena;
use mesh::Mesh;
use pipeline::Pipeline;

use crate::{handle::Handle, MeshBuilder, Vertex};


pub mod pipeline;
pub mod mesh;
pub mod instance_list;




pub struct InstanceList {
    pipeline: Handle<Pipeline>,
    mesh: Handle<Mesh>,
}

impl InstanceList {
    pub fn pipeline(&self) -> Handle<Pipeline> {
        self.pipeline
    }

    pub fn mesh(&self) -> Handle<Mesh> {
        self.mesh
    }

    pub fn new(pipeline: Handle<Pipeline>, mesh: Handle<Mesh>) -> Self {
        Self {
            pipeline,
            mesh,
        }
    }
}

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
}

impl Resources {
    pub fn new() -> Self {
        let pipelines = Arena::new();
        let meshes = Arena::new();
        let instance_lists = Arena::new();

        let pipeline_dependents = HashMap::new();
        let mesh_dependents = HashMap::new();

        Self {
            pipelines,
            meshes,
            instance_lists,
            pipeline_dependents,
            mesh_dependents,
        }
    }

    pub fn add_pipeline(
        &mut self,
    ) -> Handle<Pipeline> {
        // create and add pipeline
        let pipeline = Pipeline { };
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
    ) -> Handle<InstanceList> {
        // add instance list
        let instance_list = InstanceList::new(pipeline, mesh);
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

}

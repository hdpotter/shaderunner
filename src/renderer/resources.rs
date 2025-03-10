use std::collections::HashMap;

use instance::Instance;
use instance_list::InstanceList;
use material::Material;
use mesh::Mesh;
use slotmap::{DefaultKey, HopSlotMap, SecondaryMap, SlotMap};

use crate::{handle::Handle, MeshBuilder, Transform, Vertex};

use super::create_pipeline::create_render_pipeline;


pub mod pipeline;
pub mod mesh;
pub mod instance_list;
pub mod instance;
pub mod uniforms;
pub mod many_one;
pub mod misc;
pub mod material;
pub mod texture;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct InstanceListRef {
    material: Handle<Material>,
    instance_list: Handle<InstanceList>,
}

impl InstanceListRef {
    pub fn material(&self) -> Handle<Material> {
        self.material
    }
    
    pub fn instance_list(&self) -> Handle<InstanceList> {
        self.instance_list
    }

    pub fn new(material: Handle<Material>, instance_list: Handle<InstanceList>) -> Self {
        Self {
            material,
            instance_list,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct InstanceRef {
    instance_list_ref: InstanceListRef,
    handle: Handle<Instance>,
}

impl InstanceRef {
    pub fn instance_list_ref(&self) -> InstanceListRef {
        self.instance_list_ref
    }

    pub fn handle(&self) -> Handle<Instance> {
        self.handle
    }

    pub fn new(instance_list_ref: InstanceListRef, handle: Handle<Instance>) -> Self {
        Self {
            instance_list_ref,
            handle,
        }
    }
}

// Where should we put the dependents lists for meshes and pipelines?  The structure is simpler to
// understand if we do instance list -> mesh/pipeline links in instance list and back links as hashtables.
// We need to sort by pipeline when rendering, though, as shader switching is expensive, so in practice
// pipelines have a practical need to understand their dependents.

pub struct Resources {
    materials: HopSlotMap<DefaultKey, Material>,
    instance_lists_by_material: SecondaryMap<DefaultKey, HopSlotMap<DefaultKey, InstanceList>>,
    
    meshes: SlotMap<DefaultKey, Mesh>,
    instance_lists_by_mesh: SecondaryMap<DefaultKey, SlotMap<DefaultKey, InstanceListRef>>,

    instance_list_coverage: HashMap<(Handle<Material>, Handle<Mesh>), InstanceListRef>,

    instance_lists_to_purge: Vec<InstanceListRef>, //todo: could make another one for materials specifically to avoid double indirection

    // necessary to be able to remove an instance list from mesh dependents
    // would prefer not to store it here, but cannot store it in InstanceListRef because we need to be able
    //   to make an InstanceListRef from just material and instance handle and would prefer not to store it
    //   on InstanceList because InstanceList shouldn't know about Resources structure, and we would need to
    //   use some kind of trick to deal with the chicken-egg problem
    instance_list_mesh_backlinks: HashMap<InstanceListRef, Handle<InstanceListRef>>
}

impl Resources {

    pub fn material(&self, material: Handle<Material>) -> &Material {
        &self.materials[material]
    }

    pub fn mesh(&self, mesh: Handle<Mesh>) -> &Mesh {
        &self.meshes[mesh]
    }

    fn instance_list(&self, instance_list_ref: InstanceListRef) -> &InstanceList {
        &self.instance_lists_by_material
            [instance_list_ref.material().key()]
            [instance_list_ref.instance_list().key()]
    }

    fn instance_list_mut(&mut self, instance_list_ref: InstanceListRef) -> &mut InstanceList {
        &mut self.instance_lists_by_material
            [instance_list_ref.material().key()]
            [instance_list_ref.instance_list().key()]
    }

    pub fn new() -> Self {
        let materials = HopSlotMap::new();
        let instance_lists_by_material = SecondaryMap::new();

        let meshes = SlotMap::new();
        let instance_lists_by_mesh = SecondaryMap::new();

        let instance_list_coverage = HashMap::new();

        let instance_lists_to_purge = Vec::new();

        let instance_list_mesh_backlinks = HashMap::new();

        Self {
            materials,
            instance_lists_by_material,
            meshes,
            instance_lists_by_mesh,
            instance_list_coverage,
            instance_lists_to_purge,
            instance_list_mesh_backlinks,
        }
    }

    pub fn add_material(
        &mut self,
        color_format: wgpu::TextureFormat,
        depth_format: wgpu::TextureFormat,
        vertex_layouts: &[wgpu::VertexBufferLayout<'_>],
        pipeline_layout: &wgpu::PipelineLayout,
        shader: &wgpu::ShaderModule,
        primitive: wgpu::PrimitiveState,
        device: &wgpu::Device,
    ) -> Handle<Material> {
        // create pipeline
        let pipeline = create_render_pipeline(
            device,
            pipeline_layout,
            color_format,
            Some(depth_format),
            vertex_layouts,
            shader,
            primitive,
        );

        // create and add mesh
        let material = Material::new(pipeline);
        let handle = Handle::insert_hop(&mut self.materials, material);
        
        // add dependent list
        let map = HopSlotMap::new();
        self.instance_lists_by_material.insert(handle.key(), map);

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
        let dependents = SlotMap::new();
        self.instance_lists_by_mesh.insert(handle.key(), dependents);

        // return
        handle
    }

    fn add_instance_list(
        &mut self,
        material: Handle<Material>,
        mesh: Handle<Mesh>,
        device: &wgpu::Device,
    ) -> InstanceListRef {
        // create and add instance list
        let instance_list = InstanceList::new(mesh, material, device);

        let map = &mut self.instance_lists_by_material[material.key()];
        let instance_list = Handle::insert_hop(map, instance_list);
        let instance_list = InstanceListRef::new(material, instance_list);

        // add as mesh dependent
        let map = &mut self.instance_lists_by_mesh[mesh.key()];
        let handle = Handle::insert(map, instance_list);

        // add to coverage
        assert!(self.instance_list_coverage.insert((material, mesh), instance_list) == None);

        // add mesh backlinks
        self.instance_list_mesh_backlinks.insert(instance_list, handle);

        instance_list
    }

    fn remove_instance_list(
        &mut self,
        instance_list: InstanceListRef
    ) {
        let instance_list_struct = self.instance_list(instance_list);
        let material = instance_list_struct.material();
        let mesh = instance_list_struct.mesh();

        // remove as mesh dependent
        let handle = self.instance_list_mesh_backlinks[&instance_list];
        let map = &mut self.instance_lists_by_mesh[mesh.key()];
        map.remove(handle.key());

        // remove from mesh backlinks
        self.instance_list_mesh_backlinks.remove(&instance_list);

        // remove struct from list by material
        let map = &mut self.instance_lists_by_material[material.key()];
        map.remove(instance_list.instance_list().key());
    }

    pub fn remove_material(
        &mut self,
        material: Handle<Material>,
    ) {
        self.purge_instance_lists_material(material);

        // verify no dependents and remove dependent list
        if self.instance_lists_by_material[material.key()].len() > 0 {
            panic!("attempted to remove material with at least one dependent instance");
        }
        self.instance_lists_by_material.remove(material.key());

        // remove material
        self.materials.remove(material.key());
    }

    /// Remove any empty instance lists depending on `material`.
    fn purge_instance_lists_material(&mut self, material: Handle<Material>) {
        self.instance_lists_to_purge.clear();
        for (key, instance_list) in &self.instance_lists_by_material[material.key()] {
            if instance_list.instance_count() == 0 {
                let instance_list_ref = InstanceListRef::new(material, Handle::new(key));
                self.instance_lists_to_purge.push(instance_list_ref);
            }
        }
        for i in 0..self.instance_lists_to_purge.len() {
            self.remove_instance_list(self.instance_lists_to_purge[i]);
        }
    }

    pub fn remove_mesh(
        &mut self,
        mesh: Handle<Mesh>
    ) {
        self.purge_instance_lists_mesh(mesh);

        // verify no dependents and remove dependent list
        if self.instance_lists_by_mesh[mesh.key()].len() > 0 {
            panic!("attempted to remove mesh with at least one dependent instance list");
        }
        self.instance_lists_by_mesh.remove(mesh.key());

        // remove pipeline
        self.meshes.remove(mesh.key());
    }

    /// Remove any instance lists depending on `mesh`.
    fn purge_instance_lists_mesh(&mut self, mesh: Handle<Mesh>) {
        self.instance_lists_to_purge.clear();
        for &instance_list_ref in self.instance_lists_by_mesh[mesh.key()].values() {
            if self.instance_list(instance_list_ref).instance_count() == 0 {
                self.instance_lists_to_purge.push(instance_list_ref);
            }
        }
        for i in 0..self.instance_lists_to_purge.len() {
            self.remove_instance_list(self.instance_lists_to_purge[i]);
        }
    }



    pub fn update_instance_buffers(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        // todo: custom iterators to provide Handle
        for material in self.materials.keys() {
            for instance_list in self.instance_lists_by_material[material].values_mut() {
                instance_list.build_and_upload_instance_buffer(device, queue);
            }
        }
    }




    fn ensure_instance_list(&mut self, material: Handle<Material>, mesh: Handle<Mesh>, device: &wgpu::Device) -> InstanceListRef {
        match self.instance_list_coverage.get(&(material, mesh)) {
            Some(instance) => *instance,
            None => {
                let instance = self.add_instance_list(material, mesh, device);
                self.instance_list_coverage.insert((material, mesh), instance);

                instance
            }
        }
    }

    pub fn add_instance(&mut self, material: Handle<Material>, mesh: Handle<Mesh>, transform: Transform, device: &wgpu::Device) -> InstanceRef {
        let instance_list = self.ensure_instance_list(material, mesh, device);
        let instance = self.instance_list_mut(instance_list).add_instance(transform);
        InstanceRef::new(instance_list, instance)
    }

    pub fn update_instance(&mut self, instance: InstanceRef, transform: Transform) {
        self.instance_list_mut(instance.instance_list_ref()).update_instance(instance.handle(), transform);
    }

    pub fn set_instance_active(&mut self, instance: InstanceRef, active: bool) {
        self.instance_list_mut(instance.instance_list_ref()).set_instance_active(instance.handle(), active);
    }

    pub fn remove_instance(&mut self, instance: InstanceRef) {
        self.instance_list_mut(instance.instance_list_ref()).remove_instance(instance.handle());
    }


    pub fn iterate_instance_lists(&self, material: Handle<Material>) -> slotmap::hop::Values<DefaultKey, InstanceList> {
        self.instance_lists_by_material[material.key()].values()
    }

    pub fn iterate_instance_lists_mut(&mut self, material: Handle<Material>) -> slotmap::hop::ValuesMut<DefaultKey, InstanceList> {
        self.instance_lists_by_material[material.key()].values_mut()
    }

    pub fn iterate_materials(&self) -> slotmap::hop::Keys<DefaultKey, Material> {
        self.materials.keys()
    }
}



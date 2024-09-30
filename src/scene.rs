use std::marker::PhantomData;
use cgmath::{Vector3, Quaternion, Zero, Rotation3};
use crate::mesh::{Mesh, Vertex};

pub mod camera;
pub mod light;

#[derive(Debug, Copy, Clone)]
pub struct Transform {
    translation: Vector3<f32>,
    rotation: Quaternion<f32>,
    scale: f32,
}

impl Transform {
    pub fn translation(&self) -> Vector3<f32> {
        self.translation
    }

    pub fn rotation(&self) -> Quaternion<f32> {
        self.rotation
    }

    pub fn scale(&self) -> f32 {
        self.scale
    }

    pub fn new(translation: Vector3<f32>, rotation: Quaternion<f32>, scale: f32) -> Transform {
        Transform {
            translation,
            rotation,
            scale,
        }
    }

    pub fn from_translation(translation: Vector3<f32>) -> Transform {
        Transform {
            translation,
            rotation: Quaternion::from_angle_x(cgmath::Rad(0.0)),
            scale: 1.0,
        }
    }

    pub fn from_rotation(rotation: Quaternion<f32>) -> Transform {
        Transform {
            translation: Vector3::zero(),
            rotation,
            scale: 1.0,
        }
    }

    pub fn from_scale(scale: f32) -> Transform {
        Transform {
            translation: Vector3::zero(),
            rotation: Quaternion::from_angle_x(cgmath::Rad(0.0)),
            scale,
        }
    }

    pub fn identity() -> Transform {
        Transform {
            translation: Vector3::zero(),
            rotation: Quaternion::from_angle_x(cgmath::Rad(0.0)),
            scale: 1.0,
        }
    }
}



// a mesh, in
pub struct Model<T: Vertex> {
    transform: Transform,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,

    vertex_type_phantom: PhantomData<T>,
}

impl<T: Vertex> Model<T> {
    pub fn vertex_buffer(&self) -> &wgpu::Buffer {
        &self.vertex_buffer
    }

    pub fn index_buffer(&self) -> &wgpu::Buffer {
        &self.index_buffer
    }

    pub fn index_count(&self) -> u32 {
        self.index_count
    }

    pub fn transform(&self) -> &Transform {
        &self.transform
    }


    pub fn new(transform: Transform, mesh: Mesh<T>, device: &wgpu::Device) -> Model<T> {
        let vertex_buffer = mesh.export_vertex_buffer(device);
        let index_buffer = mesh.export_index_buffer(device);
        let index_count = mesh.index_count();

        Model {
            transform,
            vertex_buffer,
            index_buffer,
            index_count,

            vertex_type_phantom: PhantomData,
        }
    }
}



// pub struct InstanceList {
//     mesh: Mesh,
//     transforms: Vec<Transform>,

// }


// pub struct Scene {
//     camera: Camera,
//     light: DirectionalLight,
//     // model: Model<ColorNormalVertex>,
// }

// impl Scene {
//     pub fn camera(&self) -> &Camera {
//         &self.camera
//     }

//     pub fn light(&self) -> &DirectionalLight {
//         &self.light
//     }

//     // pub fn model(&self) -> &Model<ColorNormalVertex> {
//     //     &self.model
//     // }
    
//     pub fn new(
//         camera: Camera,
//         light: DirectionalLight,
//         // model: Model<ColorNormalVertex>,
//     ) -> Self {
//         Scene {
//             camera,
//             light,
//             // model,
//         }
//     }
// }
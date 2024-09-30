use cgmath::{Vector3, Point3, EuclideanSpace, InnerSpace,};
use winit::dpi::{PhysicalSize, PhysicalPosition};

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

#[rustfmt::skip]
pub const ZEROES: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    0.0, 0.0, 0.0, 0.0,
    0.0, 0.0, 0.0, 0.0,
    0.0, 0.0, 0.0, 0.0,
    0.0, 0.0, 0.0, 0.0,
);

pub struct Camera {
    eye: Vector3<f32>,
    target: Vector3<f32>,
    up: Vector3<f32>,
    aspect: f32,
    fovy: f32,
    z_near: f32,
    z_far: f32,
    dirty: bool,
}

impl Camera {
    pub fn eye(&self) -> Vector3<f32> {
        self.eye
    }

    pub fn target(&self) -> Vector3<f32> {
        self.target
    }

    pub fn up(&self) -> Vector3<f32> {
        self.up
    }

    pub fn update_eye(&mut self, eye: Vector3<f32>) {
        self.eye = eye;
        self.dirty= true;
    }

    pub fn update_target(&mut self, target: Vector3<f32>) {
        self.target = target;
        self.dirty = true;
    }

    pub fn update_up(&mut self, up: Vector3<f32>) {
        self.up = up;
        self.dirty = true;
    }

    pub fn resize(&mut self, size: &PhysicalSize<u32>) {
        self.aspect = (size.width as f32) / (size.height as f32);
    }

    pub fn new(
        eye: Vector3<f32>,
        target: Vector3<f32>,
        up: Vector3<f32>,
        aspect: f32,
        fovy: f32,
        z_near: f32,
        z_far: f32,
    ) -> Camera {
        Camera {
            eye,
            target,
            up,
            aspect,
            fovy,
            z_near,
            z_far,
            dirty: true,
        }
    }

    // todo: consider not stack allocating?
    pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at_rh(
            Point3::from_vec(self.eye), 
            Point3::from_vec(self.target.into()),
            self.up
        );
        let proj = cgmath::perspective(
            cgmath::Rad(self.fovy),
            self.aspect,
            self.z_near,
            self.z_far,
        );

        return proj * view;
        // return cgmath::Matrix4::identity();
    }

    // todo: consider using abstraction over winit data structures
    pub fn pixel_to_ray(&self, window_size: PhysicalSize<u32>, mouse_position: PhysicalPosition<f32>) -> Ray {
        // (0, 0) is in the upper left for winit mouse position
        let x = (mouse_position.x - window_size.width as f32 / 2.0) / (window_size.height as f32 / 2.0);
        let z = -(mouse_position.y - window_size.height as f32 / 2.0) / (window_size.height as f32 / 2.0);
        let y = 1.0 / f32::tan(self.fovy / 2.0);

        let camera_y_direction = (self.target() - self.eye()).normalize();
        let camera_x_direction = camera_y_direction.cross(self.up()).normalize();
        let camera_z_direction = (self.up() - self.up().dot(camera_y_direction) * camera_y_direction).normalize();

        let ray_direction = (x * camera_x_direction + y * camera_y_direction + z * camera_z_direction).normalize();
        Ray::new(
            self.eye,
            ray_direction
        )
    }
}


#[cfg(test)]
mod tests {
    use cgmath::Zero;

    use super::*;

    #[test]
    fn test_pixel_to_ray() {

        let camera = Camera::new(
            Vector3::zero(),
            Vector3::new(0.0, 1.0, 0.0),
            Vector3::new(0.0, 0.0, 1.0),
            1.25,
            std::f32::consts::TAU / 8.0,
            0.1,
            100.0,
        );

        let ray = camera.pixel_to_ray(
            PhysicalSize { width: 400, height: 300},
            PhysicalPosition { x: 400.0, y: 300.0},
        );

        println!("{:?}", ray.direction());



    }
}










#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    position: [f32; 4],
    view_proj: [[f32; 4]; 4],
}

pub fn create_camera_bind_group_and_layout(
    camera_buffer: &wgpu::Buffer,
    light_buffer: &wgpu::Buffer,
    device: &wgpu::Device
) -> (wgpu::BindGroupLayout, wgpu::BindGroup) {
    let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
        label: Some("camera bind group layout"),
    });

    let camera_bind_group = device.create_bind_group( &wgpu::BindGroupDescriptor {
        layout: &camera_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: light_buffer.as_entire_binding(),
            },
        ],
        label: Some("camera bind group"),
    });

    (camera_bind_group_layout, camera_bind_group)
}

pub struct Ray {
    source: Vector3<f32>,
    direction: Vector3<f32>,
}

impl Ray {
    pub fn source(&self) -> Vector3<f32> {
        self.source
    }

    pub fn direction(&self) -> Vector3<f32> {
        self.direction
    }

    pub fn new(source: Vector3<f32>, direction: Vector3<f32>) -> Self {
        Self {
            source,
            direction,
        }
    }
}
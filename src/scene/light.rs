use cgmath::Vector3;


pub struct DirectionalLight {
    direction: Vector3<f32>,
    color: Vector3<f32>,
    intensity: f32,
}

impl DirectionalLight {
    pub fn direction(&self) -> Vector3<f32> {
        self.direction
    }
    
    pub fn color(&self) -> Vector3<f32> {
        self.color
    }

    pub fn intensity(&self) -> f32 {
        self.intensity
    }

    pub fn new(
        direction: Vector3<f32>,
        color: Vector3<f32>,
        intensity: f32,
    ) -> Self {
        DirectionalLight {
            direction,
            color,
            intensity,
        }
    }
}

pub struct AmbientLight {
    color: Vector3<f32>,
    intensity: f32,
}

impl AmbientLight {
    pub fn color(&self) -> Vector3<f32> {
        self.color
    }

    pub fn intensity(&self) -> f32 {
        self.intensity
    }

    pub fn new(color: Vector3<f32>, intensity: f32) -> Self {
        AmbientLight {
            color,
            intensity,
        }
    }
}
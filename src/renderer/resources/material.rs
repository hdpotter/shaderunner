/// Contains all the details specifying how to render a mesh.
pub struct Material {
    // to begin with, we just support a single pipeline
    pipeline: wgpu::RenderPipeline
}

impl Material {
    pub fn pipeline(&self) -> &wgpu::RenderPipeline {
        &self.pipeline
    }

    pub fn new(pipeline: wgpu::RenderPipeline) -> Self {
        Self {
            pipeline
        }
    }
}
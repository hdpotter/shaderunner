pub struct Pipeline {
    pipeline: wgpu::RenderPipeline
}

impl Pipeline {
    pub fn pipeline(&self) -> &wgpu::RenderPipeline {
        &self.pipeline
    }

    pub fn new(pipeline: wgpu::RenderPipeline) -> Self {
        Self {
            pipeline,
        }
    }
}

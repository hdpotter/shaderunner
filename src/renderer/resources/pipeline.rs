use generational_arena::Arena;

use super::instance_list::InstanceList;

pub struct Pipeline {
    pipeline: wgpu::RenderPipeline,
    dependents: Arena<InstanceList>,
}

impl Pipeline {
    pub fn pipeline(&self) -> &wgpu::RenderPipeline {
        &self.pipeline
    }

    pub fn new(pipeline: wgpu::RenderPipeline) -> Self {
        let dependents = Arena::new();
        
        Self {
            dependents,
            pipeline,
        }
    }
}


pub struct ResizableBuffer {
    buffer: wgpu::Buffer,

    usage: wgpu::BufferUsages,

    size: u32,
}

impl ResizableBuffer {
    pub fn capacity(&self) -> u32 {
        self.buffer.size() as u32
    }

    pub fn size(&self) -> u32 {
        panic!("not supported");
        // self.size
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    pub fn new(capacity: u32, usage: wgpu::BufferUsages, device: &wgpu::Device) -> ResizableBuffer {
        let capacity = (capacity + 4 - capacity % 4) as u64; //can panic if not divisible by 4
            // (https://github.com/gfx-rs/wgpu/issues/4731)
        
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(Self::LABEL), // todo: consider storing string label in struct
            size: capacity,
            usage,
            mapped_at_creation: false,
        });

        ResizableBuffer {
            buffer,
            usage,
            size: 0,
        }
    }

    pub fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, data: &[u8]) {
        // reallocate buffer if we need to expand it
        if data.len() as u32 > self.capacity() {
            let new_capacity = data.len() + data.len() / 4;
            let new_capacity = (new_capacity + 4 - new_capacity % 4) as u64; // can panic if not divisible by 4
                // (https://github.com/gfx-rs/wgpu/issues/4731)

            self.buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(Self::LABEL),
                size: new_capacity,
                usage: self.usage,
                mapped_at_creation: false,
            });
        }

        // upate buffer buffer
        queue.write_buffer(&self.buffer, 0, data);
        self.size = data.len() as u32;
        
    }

    const LABEL: &'static str = "resizable buffer";

}
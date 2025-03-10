use image::{EncodableLayout, ImageBuffer, Rgba};

pub struct Texture {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    sampler: wgpu::Sampler,
}

impl Texture {
    pub fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }

    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    pub fn sampler(&self) -> &wgpu::Sampler {
        &self.sampler
    }

    pub fn new(
        size: wgpu::Extent3d,
        data: &[u8],
        queue: &wgpu::Queue,
        device: &wgpu::Device,
    ) -> Self {

        // create texture
        let texture = device.create_texture(
            &wgpu::TextureDescriptor {
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                label: None,
                view_formats: &[],
            }
        );

        // copy to gpu
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * size.width),
                rows_per_image: Some(size.height),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            texture,
            view,
            sampler,
        }
    }

    pub fn new_from_image(
        image: &ImageBuffer<Rgba<u8>, Vec<u8>>,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
    ) -> Self {
        let data = image.as_bytes();

        Self::new(
            wgpu::Extent3d {
                width: image.width(),
                height: image.height(),
                depth_or_array_layers: 1,
            },
            data,
            queue,
            device,
        )
    }

    pub fn new_from_procedure<F>(
        width: u32,
        height: u32,
        mut procedure: F,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
    ) -> Self 
    where F: FnMut(u32, u32) -> [u8; 4]
    {
        let mut image = ImageBuffer::new(width, height);

        for (x, y, pixel) in image.enumerate_pixels_mut() {
            *pixel = image::Rgba(procedure(x, y));
        }

        Self::new_from_image(
            &image,
            queue,
            device,
        )
    }
}


use winit::window::Window;

use crate::{scene::{Transform, camera::Camera, light::{DirectionalLight, AmbientLight}}, color_normal_vertex::ColorNormalVertex, mesh::{Vertex, Mesh}};

use self::{gpu_resources::{Resources, MeshHandle}, instances::{InstanceListResource, InstanceHandle, InstanceData}};

pub mod pipeline;

pub mod gpu_resources;
pub mod instances;
pub mod resizable_buffer;
pub mod texture;

pub struct UIFrame {
    pub clipped_primitives: Vec<egui_winit::egui::ClippedPrimitive>,
    pub textures_delta: egui::TexturesDelta
}

// impl UIFrame {
//     pub fn new(
//         clipped_primitives: Vec<egui_winit::egui::ClippedPrimitive>,
//         textures_delta: egui::TexturesDelta,
//     ) -> Self {
//         Self {
//             clipped_primitives,
//             textures_delta,
//         }
//     }

//     pub fn manage_textures(
//         &self,
//         device: &wgpu::Device,
//         queue: &wgpu::Queue,
//     ) {

//     }
// }

pub struct Renderer {
    device: wgpu::Device,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
    queue: wgpu::Queue,
    pipeline: wgpu::RenderPipeline,

    ui_renderer: egui_wgpu::Renderer,

    resources: Resources,
}

impl Renderer {
    // todo: figure out how to handle these elegantly
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    // pub fn size(&self) -> 

    pub async fn new(window: &Window) -> Renderer {
        let size = window.inner_size();

        let instance = wgpu::Instance::default();
        
        // todo: consider making this safe
        let surface = unsafe {
            instance.create_surface_unsafe(wgpu::SurfaceTargetUnsafe::from_window(&window)
                .expect("error creating surface"))
            }.unwrap();
        
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ).await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                label: None,
            },
            None,
        ).await.unwrap();
        
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        
        
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        // surface.configure(&device, &surface_config);

        let resources = Resources::new(&device, &surface_config);

        let depth_format = Some(wgpu::TextureFormat::Depth32Float);

        let pipeline = {
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("render pipeline layout"),
                bind_group_layouts: &[
                    resources.camera_bind_group_layout(),
                ],
                push_constant_ranges: &[],
            });

            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
            };

            pipeline::create_render_pipeline(
                &device,
                &layout,
                surface_config.format,
                depth_format,
                &[ColorNormalVertex::vertex_buffer_layout(), InstanceData::vertex_buffer_layout()],
                shader,
            )
        };

        let ui_renderer = egui_wgpu::Renderer::new(
            &device,
            surface_config.format,
            depth_format,
            1,
        );


        Renderer {
            device,
            surface,
            surface_config,
            queue,
            pipeline,

            ui_renderer,

            resources,
        }
    }

    pub fn resize(&mut self, new_size: &winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.surface_config.width = new_size.width;
            self.surface_config.height = new_size.height;
            self.surface.configure(&self.device, &self.surface_config);
            self.resources.resize_depth_texture(&self.device, &self.surface_config);
        }
    }

    pub fn render(&mut self) {
        self.render_with_ui(None);
    }

    pub fn render_with_ui(&mut self, ui_frame: Option<&UIFrame>) {
        // update instance buffers
        for instance_list in self.resources.iterate_instance_lists_mut() {
            instance_list.build_instance_buffer(&self.device, &self.queue);
        }
        
        let output = match self.surface.get_current_texture() {
            Ok(surface_texture) => surface_texture,
            Err(error) => {
                println!("surface error: {}", error);
                panic!();
            },
        };


        // let output = self.surface.get_current_texture().unwrap();
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("render encoder"),
        });
        
        // update ui resources
        // todo: pull this into UIManager
        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [self.surface_config.width, self.surface_config.height],
            pixels_per_point: 1.0,
        };
        if let Some(inner_ui_frame) = ui_frame {
            // texture sets
            for (texture_id, image_delta) in &inner_ui_frame.textures_delta.set {
                self.ui_renderer.update_texture(
                    &self.device,
                    &self.queue,
                    *texture_id,
                    image_delta,
                );
            }

            // texture frees
            for texture_id in &inner_ui_frame.textures_delta.free {
                self.ui_renderer.free_texture(texture_id);
            }

            // update
            self.ui_renderer.update_buffers(
                &self.device,
                &self.queue,
                &mut encoder,
                &inner_ui_frame.clipped_primitives,
                &screen_descriptor,
            );
        }

        // render pass
        {

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.01,
                            g: 0.01,
                            b: 0.01,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.resources.depth_texture_view(),
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.pipeline);

            for instance_list in self.resources.iterate_instance_lists() {
                self.draw_instance_list(&mut render_pass, instance_list, self.resources.camera_bind_group());
            }

            // draw ui
            if let Some(inner_ui_frame) = ui_frame {
               
                // render
                self.ui_renderer.render(
                    &mut render_pass,
                    &inner_ui_frame.clipped_primitives,
                    &screen_descriptor,
                );
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }

    fn draw_instance_list<'a, 'b: 'a>(
        &'b self,
        render_pass: &mut wgpu::RenderPass<'a>,
        instance_list: &'b InstanceListResource,
        camera_bind_group: &'b wgpu::BindGroup,
    ) {
        let mesh = self.resources.get_mesh(instance_list.mesh());
        render_pass.set_vertex_buffer(1, instance_list.instance_buffer().slice(..));
        render_pass.set_vertex_buffer(0, mesh.vertex_buffer().slice(..));
        render_pass.set_index_buffer(mesh.index_buffer().slice(..), wgpu::IndexFormat::Uint32);
        render_pass.set_bind_group(0, camera_bind_group, &[]);
        render_pass.draw_indexed(0..mesh.index_count(), 0, 0..instance_list.buffered_instance_count());
    }

    // ================================================================
    // interface for resources
    // ================================================================
    pub fn add_mesh(&mut self, mesh: &Mesh<ColorNormalVertex>) -> MeshHandle {
        self.resources.add_mesh(mesh, &self.device)
    }

    pub fn remove_mesh(&mut self, mesh: MeshHandle) {
        self.resources.remove_mesh(mesh);
    }

    pub fn add_instance(&mut self, mesh: MeshHandle, transform: Transform) -> InstanceHandle {
        self.resources.add_instance(mesh, transform)
    }

    pub fn update_instance(&mut self, instance: InstanceHandle, transform: Transform) {
        self.resources.update_instance(instance, transform);
    }

    pub fn set_instance_active(&mut self, instance: InstanceHandle, active: bool) {
        self.resources.set_instance_active(instance, active);
    }

    pub fn remove_instance(&mut self, instance: InstanceHandle) {
        self.resources.remove_instance(instance);
    }

    pub fn update_camera(&mut self, camera: &Camera) {
        self.resources.update_camera(camera, &self.queue);
    }

    pub fn update_light(&mut self, directional_light: &DirectionalLight, ambient_light: &AmbientLight) {
        self.resources.update_light(directional_light, ambient_light, &self.queue);
    }

}
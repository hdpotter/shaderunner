use cgmath::Vector3;
use egui::Context;
use line_renderer::LineRenderer;
use winit::window::Window;

use crate::{color_normal_vertex::ColorNormalVertex, color_vertex::ColorVertex, mesh::{Mesh, Vertex}, scene::{camera::Camera, light::{AmbientLight, DirectionalLight}, Transform}, UIManager};

use self::{gpu_resources::{Resources, MeshHandle}, instances::{InstanceListResource, InstanceHandle, InstanceData}};

pub mod create_pipeline;

pub mod gpu_resources;
pub mod line_renderer;
pub mod instances;
pub mod resizable_buffer;
pub mod texture;
pub mod pipeline;


pub struct Renderer {
    window: Window,
    device: wgpu::Device,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
    queue: wgpu::Queue,
    tri_pipeline: wgpu::RenderPipeline,
    line_pipeline: wgpu::RenderPipeline,

    line_renderer: LineRenderer,
    
    ui_manager: UIManager,

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

    pub async fn new(window: Window) -> Renderer {
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
                memory_hints: Default::default(),
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
        
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("render pipeline layout"),
            bind_group_layouts: &[
                resources.camera_bind_group_layout(),
            ],
            push_constant_ranges: &[],
        });
            
        let tri_pipeline = {
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("tri_shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
            };
            let shader = device.create_shader_module(shader);

            let tri_primitive = wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            };

            create_pipeline::create_render_pipeline(
                &device,
                &pipeline_layout,
                surface_config.format,
                depth_format,
                &[ColorNormalVertex::vertex_buffer_layout(), InstanceData::vertex_buffer_layout()],
                &shader,
                tri_primitive,
            )
        };

        let line_pipeline = {
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("line_shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("line_shader.wgsl").into()),
            };
            let shader = device.create_shader_module(shader);

            let line_primitive = wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
                front_face: wgpu::FrontFace::Ccw,
                .. Default::default()
            };

            create_pipeline::create_render_pipeline(
                &device,
                &pipeline_layout,
                surface_config.format,
                depth_format,
                &[ColorVertex::vertex_buffer_layout()],
                &shader,
                line_primitive,
            )
        };

        let line_renderer = LineRenderer::new(&device);

        let ui_manager = UIManager::new(
            &window,
            &device,
            &surface_config,
            depth_format,
        );


        Renderer {
            window,
            device,
            surface,
            surface_config,
            queue,
            tri_pipeline,
            line_pipeline,

            line_renderer,
            ui_manager,

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

        // update line renderer
        self.line_renderer.update_buffer_and_clear(&self.device, &self.queue);


        // let output = self.surface.get_current_texture().unwrap();
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("render encoder"),
        });
        
        // update ui resources
        self.ui_manager.update_resources(
            &self.device,
            &self.queue,
            &mut encoder,
            &self.surface_config,
        );

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


            // draw triangles
            render_pass.set_pipeline(&self.tri_pipeline);
            for instance_list in self.resources.iterate_instance_lists() {
                self.draw_instance_list(&mut render_pass, instance_list, self.resources.camera_bind_group());
            }

            // draw lines
            render_pass.set_pipeline(&self.line_pipeline);
            self.line_renderer.render(
                &mut render_pass,
                self.resources.camera_bind_group(),
            );
            

            // draw ui
            self.ui_manager.render(&mut render_pass.forget_lifetime()); // egui makes us forget lifetime
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }

    fn draw_instance_list(
        &self,
        render_pass: &mut wgpu::RenderPass,
        instance_list: &InstanceListResource,
        camera_bind_group: &wgpu::BindGroup,
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

    // ================================================================
    // immediate mode line drawing
    // ================================================================
    
    ///Draws a line in immediate mode - i.e., this function draws a line on the next `render()` call only.
    pub fn draw_line(&mut self, start: ColorVertex, end: ColorVertex) {
        self.line_renderer.draw_line(start, end);
    }

    ///Draws a red line in immediate mode - i.e., this function draws a line on the next `render()` call only.
    pub fn draw_line_red(&mut self, start: Vector3<f32>, end: Vector3<f32>) {
        let red = Vector3::new(1_f32, 0_f32, 0_f32);
        let start = ColorVertex::new(start, red);
        let end = ColorVertex::new(end, red);
        self.draw_line(start, end);
    }

    ///Draws a green line in immediate mode - i.e., this function draws a line on the next `render()` call only.
    pub fn draw_line_green(&mut self, start: Vector3<f32>, end: Vector3<f32>) {
        let green = Vector3::new(0_f32, 1_f32, 0_f32);
        let start = ColorVertex::new(start, green);
        let end = ColorVertex::new(end, green);
        self.draw_line(start, end);
    }

    ///Draws a blue line in immediate mode - i.e., this function draws a line on the next `render()` call only.
    pub fn draw_line_blue(&mut self, start: Vector3<f32>, end: Vector3<f32>) {
        let blue = Vector3::new(0_f32, 0_f32, 1_f32);
        let start = ColorVertex::new(start, blue);
        let end = ColorVertex::new(end, blue);
        self.draw_line(start, end);
    }

    // ================================================================
    // immediate mode gui
    // ================================================================

    pub fn run_ui<F: FnMut(&Context)>(&mut self, gui: F) {
        self.ui_manager.run(&self.window, gui);
    }

}
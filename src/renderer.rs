use cgmath::{Vector2, Vector3};
use egui::Context;
use image::ImageBuffer;
use line_renderer::LineRenderer;
use resources::{instance::InstanceData, instance_list::InstanceList, material::Material, mesh::Mesh, misc::Misc, texture::Texture, InstanceRef};
use texture_renderer::TextureRenderer;
use winit::window::Window;

use crate::{color_normal_vertex::ColorNormalVertex, color_vertex::ColorVertex, handle::Handle, mesh_builder::{MeshBuilder, Vertex}, scene::{camera::Camera, light::{AmbientLight, DirectionalLight}, Transform}, UIManager};

use self::resources::Resources;

pub mod create_pipeline;

pub mod resources;
pub mod resizable_buffer;
pub mod texture;
pub mod line_renderer;
pub mod texture_renderer;


pub struct Renderer {
    window: Window,
    device: wgpu::Device,

    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
    depth_format: wgpu::TextureFormat,

    queue: wgpu::Queue,

    line_renderer: LineRenderer,
    texture_renderer: TextureRenderer,
    ui_manager: UIManager,

    resources: Resources,
    misc: Misc,
}

impl Renderer {
    pub fn window(&self) -> &Window {
        &self.window
    }

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

        // let resources = Resources::new(&device, &surface_config);
        let resources = Resources::new();
        let misc = Misc::new(&surface_config, &device);

        let depth_format = wgpu::TextureFormat::Depth32Float;

        let line_renderer = LineRenderer::new(
            &device,
            misc.pipeline_layout(),
            surface_config.format,
            depth_format,
        );

        let texture_renderer = TextureRenderer::new(
            &device,
            misc.texture_bind_group_layout(),
            surface_config.format,
            depth_format,
        );

        let ui_manager = UIManager::new(
            &window,
            &device,
            &surface_config,
            Some(depth_format),
        );


        Renderer {
            window,
            device,

            surface,
            surface_config,
            depth_format,

            queue,

            line_renderer,
            texture_renderer,
            ui_manager,

            resources,
            misc,
        }
    }

    pub fn resize(&mut self, new_size: &winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.surface_config.width = new_size.width;
            self.surface_config.height = new_size.height;
            self.surface.configure(&self.device, &self.surface_config);
            self.misc.resize_depth_texture(&self.device, &self.surface_config);
        }
    }

    pub fn egui_event(&mut self, event: &winit::event::WindowEvent) -> bool {
        self.ui_manager.on_window_event(&self.window, event)
    }

    pub fn render(&mut self) {

        // update instance buffers
        self.resources.update_instance_buffers(&self.device, &self.queue);
        
        // get output texture
        let output = match self.surface.get_current_texture() {
            Ok(surface_texture) => surface_texture,
            Err(error) => {
                println!("surface error: {}", error);
                panic!();
            },
        };

        // update line renderer and texture renderer
        self.line_renderer.update_buffer(&self.device, &self.queue);
        self.texture_renderer.update_buffer(&self.device, &self.queue);

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
                    view: &self.misc.depth_texture_view(),
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            for material in self.resources.iterate_materials() {
                let material_handle = Handle::new(material); //todo: proper handle iteration and reconsider indirection
                let material = self.resources.material(material_handle);

                render_pass.set_pipeline(material.pipeline());

                for instance_list in self.resources.iterate_instance_lists(material_handle) {
                    if instance_list.instance_count() > 0 {
                        self.draw_instance_list(&mut render_pass, instance_list, self.misc.camera_bind_group());
                    }
                }
            }

            // draw lines
            self.line_renderer.render_and_clear(
                &mut render_pass,
                self.misc.camera_bind_group(),
            );

            // draw textures
            self.texture_renderer.render_and_clear(
                &mut render_pass,
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
        instance_list: &InstanceList,
        camera_bind_group: &wgpu::BindGroup,
    ) {
        match self.resources.mesh(instance_list.mesh()) {
            Mesh::Nonempty(mesh) => {
                render_pass.set_vertex_buffer(1, instance_list.instance_buffer().slice(..));
                render_pass.set_vertex_buffer(0, mesh.vertex_buffer().slice(..));
                render_pass.set_index_buffer(mesh.index_buffer().slice(..), wgpu::IndexFormat::Uint32);
                render_pass.set_bind_group(0, camera_bind_group, &[]);
                render_pass.draw_indexed(0..mesh.index_count(), 0, 0..instance_list.buffered_instance_count());
            },
            Mesh::Empty => (),
        }
    }

    // ================================================================
    // interface for resources
    // ================================================================
    pub fn add_mesh(&mut self, mesh: &MeshBuilder<ColorNormalVertex>) -> Handle<Mesh> {
        self.resources.add_mesh(mesh, &self.device)
    }

    pub fn add_material(
        &mut self,
        shader: wgpu::ShaderSource,
        primitive: wgpu::PrimitiveState,
    ) -> Handle<Material> {
        let shader = wgpu::ShaderModuleDescriptor {
            label: None,
            source: shader,
        };
        let shader = self.device.create_shader_module(shader); //todo: add separate shader so we can separate pipeline creation from shader creation

        self.resources.add_material(
            self.surface_config.format,
            self.depth_format,
            &[ColorNormalVertex::vertex_buffer_layout(), InstanceData::vertex_buffer_layout()],
            self.misc.pipeline_layout(),
            &shader,
            primitive,
            &self.device,
        )
    }

    pub fn remove_mesh(&mut self, mesh: Handle<Mesh>) {
        self.resources.remove_mesh(mesh);
    }

    pub fn add_texture(&mut self, texture: &ImageBuffer<image::Rgba<u8>, Vec<u8>>) -> Handle<Texture> {
        self.resources.add_texture(texture, &self.device, &self.queue)
    }

    pub fn add_instance(&mut self, material: Handle<Material>, mesh: Handle<Mesh>, transform: Transform) -> InstanceRef {
        self.resources.add_instance(material, mesh, transform, &self.device)
    }

    pub fn update_instance(&mut self, instance: InstanceRef, transform: Transform) {
        self.resources.update_instance(instance, transform);
    }

    pub fn set_instance_active(&mut self, instance: InstanceRef, active: bool) {
        self.resources.set_instance_active(instance, active);
    }

    pub fn remove_instance(&mut self, instance: InstanceRef) {
        self.resources.remove_instance(instance);
    }

    pub fn update_camera(&mut self, camera: &Camera) {
        self.misc.update_camera(camera, &self.queue);
    }

    pub fn update_light(&mut self, directional_light: &DirectionalLight, ambient_light: &AmbientLight) {
        self.misc.update_light(directional_light, ambient_light, &self.queue);
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
    // immediate mode texture drawing
    // ================================================================

    pub fn draw_texture(
        &mut self,
        texture: Handle<Texture>,
        lower_left: Vector2<f32>,
        upper_right: Vector2<f32>,
    ) {
        self.texture_renderer.draw_texture(
            texture,
            lower_left,
            upper_right,
            &self.device,
            self.misc.texture_bind_group_layout(),
            &self.resources,
        );
    }

    // ================================================================
    // immediate mode gui
    // ================================================================

    pub fn run_ui<F: FnMut(&Context)>(&mut self, gui: F) {
        self.ui_manager.run(&self.window, gui);
    }

}
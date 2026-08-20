// ------------------------------------------------------------------- imports

use anyhow::Context;

use std::sync::Arc;

use winit::event_loop::ActiveEventLoop;
use winit::window::Window;

use std::borrow::Cow;

use glam::{Mat4, Vec3, camera};

use crate::{
    config::{LOGIC_HEIGHT, LOGIC_WIDTH},
    ecs::Size,
};

mod bindgroups;
mod buffers;
mod init;
mod pipelines;
mod renderpass;
mod textures;

pub use buffers::{InstanceData, Uniforms, Vertex};

// ------------------------------------------------------------------- renderer

pub struct Renderer {
    // --------------------------------------------------------------- init
    window: Arc<Window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    is_surface_configured: bool,

    // --------------------------------------------------------------- buffers
    uniform_buffer: wgpu::Buffer,
    fullscreen_vertex_buffer: wgpu::Buffer,
    rect_vertex_buffer: wgpu::Buffer,

    instance_buffer: wgpu::Buffer,
    instances: Vec<InstanceData>,

    index_buffer: wgpu::Buffer,
    num_indices: u32,

    // --------------------------------------------------------------- textures
    mid_texture_view: wgpu::TextureView,

    // --------------------------------------------------------------- bind groups
    mid_bind_group: wgpu::BindGroup,
    scaler_bind_group: wgpu::BindGroup,

    // --------------------------------------------------------------- pipelines
    mid_render_pipeline: wgpu::RenderPipeline,
    scaler_render_pipeline: wgpu::RenderPipeline,
}

impl Renderer {
    pub async fn new(
        window: Arc<Window>,
        event_loop: &ActiveEventLoop,
        instances: &[InstanceData],
    ) -> anyhow::Result<Self> {
        // ----------------------------------------------------------- init wgpu

        let (adapter, device, queue, surface, config) = init::init_wgpu(&window, event_loop)
            .await
            .context("failed to initialise wgpu")?;

        // ----------------------------------------------------------- samplers

        let sampler_nearest = init::create_sampler(
            &device,
            wgpu::FilterMode::Nearest,
            wgpu::MipmapFilterMode::Nearest,
        );

        // ----------------------------------------------------------- buffers

        let uniform_buffer = buffers::create_uniform_buffer(&device);

        let fullscreen_vertex_buffer = buffers::create_vertex_buffer(
            &device,
            "full screen vertex buffer",
            buffers::FULLSCREEN_VERTICES,
        );

        let rect_vertex_buffer = buffers::create_vertex_buffer(
            &device,
            "rectangle vertex buffer",
            buffers::RECT_VERTICES,
        );

        let instance_buffer = buffers::create_instance_buffer(&device, instances);
        let instances = instances.to_vec();

        let index_buffer = buffers::create_index_buffer(&device, buffers::RECT_INDICES);
        let num_indices = u32::try_from(buffers::RECT_INDICES.len()).context("conversion error")?;

        // ----------------------------------------------------------- bind group layouts

        // let u_bind_group_layout = bindgroups::create_u_bind_group_layout(&device);

        let t_s_bind_group_layout = bindgroups::create_t_s_bind_group_layout(&device);

        let u_t_s_bind_group_layout = bindgroups::create_u_t_s_bind_group_layout(&device);

        // let t_t_s_bind_group_layout = bindgroups::create_t_t_s_bind_group_layout(&device);

        // ----------------------------------------------------------- load shaders

        let mid_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("mid shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("../shaders/mid.wgsl"))),
        });

        let scaler_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("scaler shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("../shaders/scaler.wgsl"))),
        });

        // --------------------------------------------------------------------------- load image

        let diffuse_bytes = include_bytes!("../../img/flowers.png");

        let diffuse_texture_view =
            textures::create_diffuse_texture(&device, &queue, "diffuse texture", diffuse_bytes)?;

        // ----------------------------------------------------------- mid

        let mid_texture_view = textures::create_texture(
            &device,
            "mid texture",
            &textures::TextureSize {
                width: u32::from(LOGIC_WIDTH),
                height: u32::from(LOGIC_HEIGHT),
            },
        );

        let mid_bind_group = bindgroups::create_u_t_s_bind_group(
            &device,
            "mid bind group",
            &u_t_s_bind_group_layout,
            &uniform_buffer,
            &diffuse_texture_view,
            &sampler_nearest,
        );

        let mid_render_pipeline = pipelines::create_pipeline_with_instance(
            &device,
            "render pipeline for mid",
            Some(&u_t_s_bind_group_layout),
            &mid_shader,
            wgpu::BlendState::ALPHA_BLENDING,
        );

        // ----------------------------------------------------------- scaler

        let scaler_bind_group = bindgroups::create_t_s_bind_group(
            &device,
            "scaler bind group",
            &t_s_bind_group_layout,
            &mid_texture_view,
            &sampler_nearest,
        );

        let scaler_render_pipeline = pipelines::create_surface_pipeline(
            &adapter,
            &device,
            &surface,
            &t_s_bind_group_layout,
            &scaler_shader,
        );

        // ----------------------------------------------------------- renderer

        let renderer = Self {
            window,
            device,
            queue,
            surface,
            config,
            is_surface_configured: false,

            fullscreen_vertex_buffer,
            rect_vertex_buffer,

            uniform_buffer,

            instance_buffer,
            instances,

            index_buffer,
            num_indices,

            mid_texture_view,

            mid_bind_group,
            scaler_bind_group,

            mid_render_pipeline,
            scaler_render_pipeline,
        };

        Ok(renderer)
    }

    pub fn render(&self) -> anyhow::Result<()> {
        if !self.is_surface_configured {
            return Ok(());
        }
        // no render unless the surface is configured

        // ----------------------------------------------------------- surface texture view

        let surface_texture = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(surface_texture)
            | wgpu::CurrentSurfaceTexture::Suboptimal(surface_texture) => surface_texture,
            wgpu::CurrentSurfaceTexture::Timeout
            | wgpu::CurrentSurfaceTexture::Occluded
            | wgpu::CurrentSurfaceTexture::Validation => {
                return Ok(()); // skip this frame
            }
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.surface.configure(&self.device, &self.config);
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Lost => {
                anyhow::bail!("lost device");
            }
        };

        let surface_texture_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        renderpass::execute_renderpasses(
            // needed for execution
            &self.device,
            &self.queue,
            surface_texture,
            // mid renderpass
            &self.mid_texture_view,
            &self.mid_render_pipeline,
            &self.mid_bind_group,
            // scaler renderpass
            &surface_texture_view,
            &self.scaler_render_pipeline,
            &self.scaler_bind_group,
            // buffers
            &self.fullscreen_vertex_buffer,
            &self.rect_vertex_buffer,
            &self.instance_buffer,
            &self.index_buffer,
            self.instances.len() as u32,
            self.num_indices,
            // window size
            Size {
                w: self.window.inner_size().width as f32,
                h: self.window.inner_size().height as f32,
            },
        )
        .context("failed to execute renderpasses")?;

        Ok(())
    }

    // --------------------------------------------------------------- get window for App

    pub fn get_window(&self) -> &Window {
        &self.window
    }

    // --------------------------------------------------------------- resize surface

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.is_surface_configured = true;
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    // --------------------------------------------------------------- update uniform buffer

    pub fn update(&self, instances: &[InstanceData]) {
        let mut view = Mat4::IDENTITY;
        view *= Mat4::from_scale(Vec3::splat(1.0));

        let projection = camera::lh::proj::directx::orthographic(
            0.0,
            f32::from(LOGIC_WIDTH),
            f32::from(LOGIC_HEIGHT),
            0.0,
            -1.0,
            1.0,
        );

        self.queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[Uniforms {
                view_matrix: view.to_cols_array_2d(),
                projection_matrix: projection.to_cols_array_2d(),
            }]),
        );

        self.queue
            .write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(instances));
    }
}

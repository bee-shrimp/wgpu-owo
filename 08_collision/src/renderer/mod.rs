// ---------------------------------------------------------------- imports

use color_eyre::eyre::{self, Result, WrapErr};

use std::borrow::Cow;
use std::sync::Arc;

use winit::event_loop::ActiveEventLoop;
use winit::window::Window;

use crate::config::{LOGIC_HEIGHT, LOGIC_WIDTH};

mod bindgroups;
mod buffers;
mod init;
mod pipelines;
mod renderpass;
mod textures;

// accessible from other modules.
pub use buffers::InstanceData;

// ---------------------------------------------------------------- renderer

pub struct Renderer {
    // ------------------------------------------------------------ init
    window: Arc<Window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    is_surface_configured: bool,

    // ------------------------------------------------------------ buffers
    uniform_buffer: wgpu::Buffer,
    fullscreen_vertex_buffer: wgpu::Buffer,
    rect_vertex_buffer: wgpu::Buffer,

    instance_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,

    num_instances: u32,

    // ------------------------------------------------------------ textures
    mid_texture_view: wgpu::TextureView,

    // ------------------------------------------------------------ bind groups
    mid_bind_group: wgpu::BindGroup,
    scaler_bind_group: wgpu::BindGroup,

    // ------------------------------------------------------------ pipelines
    mid_render_pipeline: wgpu::RenderPipeline,
    scaler_render_pipeline: wgpu::RenderPipeline,
}

impl Renderer {
    pub async fn new(
        window: Arc<Window>,
        event_loop: &ActiveEventLoop,
        instances: &[InstanceData],
    ) -> Result<Self> {
        // -------------------------------------------------------- init wgpu

        let (adapter, device, queue, surface, config) = init::init_wgpu(&window, event_loop)
            .await
            .context("failed to initialise wgpu")?;

        // -------------------------------------------------------- samplers

        let sampler_nearest = init::create_sampler(
            &device,
            wgpu::AddressMode::ClampToEdge,
            wgpu::FilterMode::Nearest,
            wgpu::MipmapFilterMode::Nearest,
        );

        // -------------------------------------------------------- buffers

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

        let instance_buffer = buffers::create_instance_buffer(&device)?;
        let num_instances = u32::try_from(instances.len())?;

        let index_buffer = buffers::create_index_buffer(&device, buffers::RECT_INDICES);

        // -------------------------------------------------------- bind group layouts

        // let u_bind_group_layout = bindgroups::create_u_bind_group_layout(&device);

        let t_s_bind_group_layout = bindgroups::create_t_s_bind_group_layout(&device);

        let u_t_s_bind_group_layout = bindgroups::create_u_t_s_bind_group_layout(&device);

        // let t_t_s_bind_group_layout = bindgroups::create_t_t_s_bind_group_layout(&device);

        // -------------------------------------------------------- load shaders

        let mid_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("mid shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("../shaders/mid.wgsl"))),
        });

        let scaler_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("scaler shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("../shaders/scaler.wgsl"))),
        });

        // -------------------------------------------------------- load image

        let diffuse_bytes = include_bytes!("../../img/robot_spritesheet.png");

        let diffuse_texture_view =
            textures::create_diffuse_texture(&device, &queue, "diffuse texture", diffuse_bytes)?;

        // -------------------------------------------------------- mid

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
        )?;

        // -------------------------------------------------------- scaler

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
        )?;

        // -------------------------------------------------------- renderer

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
            index_buffer,

            num_instances,

            mid_texture_view,

            mid_bind_group,
            scaler_bind_group,

            mid_render_pipeline,
            scaler_render_pipeline,
        };

        Ok(renderer)
    }

    pub fn render(&self) -> Result<()> {
        // no render unless the surface is configured
        if !self.is_surface_configured {
            return Ok(());
        }

        // -------------------------------------------------------- surface texture view

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
                eyre::bail!("lost device");
            }
        };

        let surface_texture_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // -------------------------------------------------------- execute renderpasses

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("render encoder"),
            });

        renderpass::draw_mid_renderpass(
            &mut encoder,
            &self.mid_texture_view,
            &self.mid_render_pipeline,
            &self.mid_bind_group,
            &self.rect_vertex_buffer,
            &self.instance_buffer,
            &self.index_buffer,
            self.num_instances,
        )?;

        renderpass::draw_scaler_renderpass(
            &mut encoder,
            &surface_texture_view,
            &self.scaler_render_pipeline,
            &self.scaler_bind_group,
            &self.fullscreen_vertex_buffer,
            self.window.inner_size(),
        );

        self.queue.submit([encoder.finish()]);
        self.queue.present(surface_texture);

        Ok(())
    }

    // ------------------------------------------------------------ get window for App

    pub fn get_window(&self) -> &Window {
        &self.window
    }

    // ------------------------------------------------------------ resize surface

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.is_surface_configured = true;
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    // ------------------------------------------------------------ update uniform buffer

    pub fn update(&mut self, instances: &[InstanceData]) -> Result<()> {
        buffers::update_uniform_buffer(&self.queue, &self.uniform_buffer);

        buffers::update_instance_buffer(&self.queue, &self.instance_buffer, instances);

        self.num_instances = u32::try_from(instances.len())?;
        Ok(())
    }
}

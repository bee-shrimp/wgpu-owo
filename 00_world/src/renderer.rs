use std::borrow::Cow;
use std::mem;
use wgpu::util::DeviceExt;

use crate::Arc;
use crate::Window;

use image::GenericImageView;

// ----------------------------------------------------------------------------------- logical size of pixel art
const LOGIC_WIDTH: u32 = 1920 / 3;
const LOGIC_HEIGHT: u32 = 1080 / 3;

// ----------------------------------------------------------------------------------- texture size for create_texture()
struct Size {
    width: u32,
    height: u32,
}

// ----------------------------------------------------------------------------------- struct for uniform buffer
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Uniforms {
    time: f32,
    input: f32,
}

// ----------------------------------------------------------------------------------- struct for vertex buffer
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
    uv: [f32; 2],
}

// ----------------------------------------------------------------------------------- descriptor for VertexBufferLayout
impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2];
    // 0 => Vertex::position, 1 => Vertex::uv

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

// ----------------------------------------------------------------------------------- fullscreen triangle
const FULLSCREEN_VERTICES: &[Vertex] = &[
    Vertex {
        position: [-1.0, -1.0], // bottom left
        uv: [0.0, 1.0],
    },
    Vertex {
        position: [-1.0, 3.0], // top left outside
        uv: [0.0, -1.0],
    },
    Vertex {
        position: [3.0, -1.0], // bottom right outside
        uv: [2.0, 1.0],
    },
];

// ----------------------------------------------------------------------------------- renderer state
pub struct Renderer {
    // surface: wgpu::Surface<'static>,
    // device: wgpu::Device,
    // queue: wgpu::Queue,
    // config: wgpu::SurfaceConfiguration,
    // is_surface_configured: bool,
    // window: Arc<Window>,
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    is_surface_configured: bool,
    // instance: wgpu::Instance,
    // size: winit::dpi::PhysicalSize<u32>,
    // surface_format: wgpu::TextureFormat,

    // ------------------------------------------------------------------------------- buffers
    uniform_buffer: wgpu::Buffer,
    fullscreen_vertex_buffer: wgpu::Buffer,

    // ------------------------------------------------------------------------------- textures
    base1_texture_view: wgpu::TextureView,
    base2_texture_view: wgpu::TextureView,
    cloud_effect_texture_view: wgpu::TextureView,

    // ------------------------------------------------------------------------------- bind groups
    diffuse1_bind_group: wgpu::BindGroup,
    diffuse2_bind_group: wgpu::BindGroup,
    cloud_effect_bind_group: wgpu::BindGroup,
    scaler_bind_group: wgpu::BindGroup,

    // ------------------------------------------------------------------------------- pipelines
    base1_render_pipeline: wgpu::RenderPipeline,
    base2_render_pipeline: wgpu::RenderPipeline,
    cloud_effect_render_pipeline: wgpu::RenderPipeline,
    scaler_render_pipeline: wgpu::RenderPipeline,
}

// ----------------------------------------------------------------------------------- renderer state
impl Renderer {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Renderer> {
        // --------------------------------------------------------------------------- size of window
        let size = window.inner_size();

        // --------------------------------------------------------------------------- create a new wgpu instance
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            flags: Default::default(),
            memory_budget_thresholds: Default::default(),
            backend_options: Default::default(),
            display: None,
        });

        // --------------------------------------------------------------------------- surface to draw onto
        let surface = instance.create_surface(window.clone()).unwrap();

        // --------------------------------------------------------------------------- physical device
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
                apply_limit_buckets: true,
            })
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
            color_space: wgpu::SurfaceColorSpace::Auto,
        };

        // --------------------------------------------------------------------------- logical device
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .unwrap();

        // --------------------------------------------------------------------------- load shaders
        let base_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shaders/base.wgsl"))),
        });

        let cloud_effect_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shaders/cloud.wgsl"))),
        });

        let scaler_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shaders/scaler.wgsl"))),
        });

        // --------------------------------------------------------------------------- uniform buffer
        let initial_uniforms = Uniforms {
            time: 0.0,
            input: 0.0,
        };

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("uniform buffer"),
            contents: bytemuck::cast_slice(&[initial_uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // --------------------------------------------------------------------------- full screen vertex buffer
        let fullscreen_vertex_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("full screen vertex buffer"),
                contents: bytemuck::cast_slice(FULLSCREEN_VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            });

        // --------------------------------------------------------------------------- samplers
        let sampler_nearest = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let sampler_linear = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });

        // --------------------------------------------------------------------------- bind group layouts
        let simple_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("simple bind group layout for sampling and drawing"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let effect_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("effect bind group layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(
                                std::mem::size_of::<Uniforms>() as u64,
                            ),
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        // --------------------------------------------------------------------------- load image
        let diffuse1_bytes = include_bytes!("../img/sea_layer.png");

        let diffuse1_texture_view =
            create_diffuse_texture(&device, &queue, "diffuse1 texture", diffuse1_bytes);

        let diffuse1_bind_group = create_simple_bind_group(
            &device,
            "diffuse1 bind group",
            &simple_bind_group_layout,
            &diffuse1_texture_view,
            &sampler_nearest,
        );

        let diffuse2_bytes = include_bytes!("../img/cloud_layer.png");

        let diffuse2_texture_view =
            create_diffuse_texture(&device, &queue, "diffuse2 texture", diffuse2_bytes);

        let diffuse2_bind_group = create_simple_bind_group(
            &device,
            "diffuse2 bind group",
            &simple_bind_group_layout,
            &diffuse2_texture_view,
            &sampler_linear,
        );

        // --------------------------------------------------------------------------- base texture
        let base1_texture_view = create_texture(
            &device,
            "base1 texture",
            &Size {
                width: LOGIC_WIDTH,
                height: LOGIC_HEIGHT,
            },
        );
        let base2_texture_view = create_texture(
            &device,
            "base2 texture",
            &Size {
                width: LOGIC_WIDTH,
                height: LOGIC_HEIGHT,
            },
        );

        // --------------------------------------------------------------------------- effect texture/bind group
        let cloud_effect_texture_view = create_texture(
            &device,
            "cloud_effect texture",
            &Size {
                width: LOGIC_WIDTH,
                height: LOGIC_HEIGHT,
            },
        );

        let cloud_effect_bind_group = create_effect_bind_group(
            &device,
            "cloud effect bind group",
            &effect_bind_group_layout,
            &uniform_buffer,
            &base2_texture_view,
            &sampler_linear,
        );

        // --------------------------------------------------------------------------- scaler bind group
        let scaler_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("bind group layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let scaler_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("scaler bind group"),
            layout: &scaler_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&base1_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&cloud_effect_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler_nearest),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&sampler_linear),
                },
            ],
        });

        // --------------------------------------------------------------------------- base pipeline
        let base1_render_pipeline = create_pipeline(
            &device,
            "render pipeline for base1",
            &simple_bind_group_layout,
            &base_shader,
            wgpu::BlendState::REPLACE,
        );

        let base2_render_pipeline = create_pipeline(
            &device,
            "render pipeline for base2",
            &simple_bind_group_layout,
            &base_shader,
            wgpu::BlendState::REPLACE,
        );

        // --------------------------------------------------------------------------- effect pipeline
        let cloud_effect_render_pipeline = create_pipeline(
            &device,
            "render pipeline for cloud_effect",
            &effect_bind_group_layout,
            &cloud_effect_shader,
            wgpu::BlendState::ALPHA_BLENDING,
        );

        // --------------------------------------------------------------------------- pipeline for scaler
        let scaler_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("render pipeline layout for surface"),
                bind_group_layouts: &[Some(&scaler_bind_group_layout)],
                immediate_size: 0,
            });

        let swapchain_capabilities = surface.get_capabilities(&adapter);
        let swapchain_format = swapchain_capabilities.formats[0];

        let scaler_render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("render pipeline for surface"),
                layout: Some(&scaler_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &scaler_shader,
                    entry_point: Some("vs_main"),
                    buffers: &[Some(Vertex::desc())],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &scaler_shader,
                    entry_point: Some("fs_main"),
                    compilation_options: Default::default(),
                    targets: &[Some(swapchain_format.into())],
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview_mask: None,
                cache: None,
            });

        // --------------------------------------------------------------------------- renderer

        let renderer = Renderer {
            window,
            device,
            queue,
            surface,
            is_surface_configured: false,
            config,

            fullscreen_vertex_buffer,

            uniform_buffer,

            base1_texture_view,
            base2_texture_view,
            cloud_effect_texture_view,
            diffuse1_bind_group,
            diffuse2_bind_group,
            cloud_effect_bind_group,
            scaler_bind_group,

            base1_render_pipeline,
            base2_render_pipeline,
            cloud_effect_render_pipeline,
            scaler_render_pipeline,
        };

        Ok(renderer)
    }

    // fn configure_surface(&self) {
    //     let surface_config = wgpu::SurfaceConfiguration {
    //         usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
    //         format: self.surface_format,
    //         view_formats: vec![self.surface_format.add_srgb_suffix()],
    //         color_space: wgpu::SurfaceColorSpace::Auto,
    //         alpha_mode: wgpu::CompositeAlphaMode::Auto,
    //         width: self.size.width,
    //         height: self.size.height,
    //         desired_maximum_frame_latency: 2,
    //         present_mode: wgpu::PresentMode::AutoVsync,
    //     };
    //     self.surface.configure(&self.device, &surface_config);
    // }

    pub fn render(&mut self) -> anyhow::Result<()> {
        self.window.request_redraw();

        // can't render unless the surface is configured
        if !self.is_surface_configured {
            return Ok(());
        }

        // --------------------------------------------------------------------------- surface texture view

        let surface_texture = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(surface_texture) => surface_texture,
            wgpu::CurrentSurfaceTexture::Suboptimal(surface_texture) => surface_texture,
            wgpu::CurrentSurfaceTexture::Timeout
            | wgpu::CurrentSurfaceTexture::Occluded
            | wgpu::CurrentSurfaceTexture::Validation => {
                // Skip this frame
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.surface.configure(&self.device, &self.config);
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Lost => {
                anyhow::bail!("lost device");
                // could recreate the devices and all resources created with it here
            }
        };

        let surface_texture_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("render encoder"),
            });

        // --------------------------------------------------------------------------- base1 renderpass
        let mut base1_renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("base1 renderpass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.base1_texture_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        // --------------------------------------------------------------------------- use the renderpass
        base1_renderpass.set_pipeline(&self.base1_render_pipeline);
        base1_renderpass.set_bind_group(0, Some(&self.diffuse1_bind_group), &[]);
        base1_renderpass.set_vertex_buffer(0, self.fullscreen_vertex_buffer.slice(..));
        base1_renderpass.set_viewport(0.0, 0.0, LOGIC_WIDTH as f32, LOGIC_HEIGHT as f32, 0.0, 1.0);
        base1_renderpass.draw(0..3, 0..1);

        // --------------------------------------------------------------------------- end the renderpass
        drop(base1_renderpass);

        // --------------------------------------------------------------------------- base2 renderpass
        let mut base2_renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("base2 renderpass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.base2_texture_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        // --------------------------------------------------------------------------- use the renderpass
        base2_renderpass.set_pipeline(&self.base2_render_pipeline);
        base2_renderpass.set_bind_group(0, Some(&self.diffuse2_bind_group), &[]);
        base2_renderpass.set_vertex_buffer(0, self.fullscreen_vertex_buffer.slice(..));
        base2_renderpass.set_viewport(0.0, 0.0, LOGIC_WIDTH as f32, LOGIC_HEIGHT as f32, 0.0, 1.0);
        base2_renderpass.draw(0..3, 0..1);

        // --------------------------------------------------------------------------- end the renderpass
        drop(base2_renderpass);

        // --------------------------------------------------------------------------- cloud_effect renderpass
        let mut cloud_effect_renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("cloud_effect renderpass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.cloud_effect_texture_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        // --------------------------------------------------------------------------- use the renderpass
        cloud_effect_renderpass.set_pipeline(&self.cloud_effect_render_pipeline);
        cloud_effect_renderpass.set_bind_group(0, Some(&self.cloud_effect_bind_group), &[]);
        cloud_effect_renderpass.set_vertex_buffer(0, self.fullscreen_vertex_buffer.slice(..));
        cloud_effect_renderpass.set_viewport(
            0.0,
            0.0,
            LOGIC_WIDTH as f32,
            LOGIC_HEIGHT as f32,
            0.0,
            1.0,
        );
        cloud_effect_renderpass.draw(0..3, 0..1);

        // --------------------------------------------------------------------------- end the renderpass
        drop(cloud_effect_renderpass);

        // --------------------------------------------------------------------------- surface renderpass
        let mut scaler_renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("scaler renderpass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &surface_texture_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        let viewport_xywh = self.calc_ratio();

        // --------------------------------------------------------------------------- use the renderpass
        scaler_renderpass.set_pipeline(&self.scaler_render_pipeline);
        scaler_renderpass.set_bind_group(0, Some(&self.scaler_bind_group), &[]);
        scaler_renderpass.set_vertex_buffer(0, self.fullscreen_vertex_buffer.slice(..));
        scaler_renderpass.set_viewport(
            viewport_xywh[0],
            viewport_xywh[1],
            viewport_xywh[2],
            viewport_xywh[3],
            0.0,
            1.0,
        );
        scaler_renderpass.draw(0..3, 0..1);

        // --------------------------------------------------------------------------- end the renderpass
        drop(scaler_renderpass);

        // --------------------------------------------------------------------------- submit the command
        self.queue.submit([encoder.finish()]);
        self.queue.present(surface_texture);

        Ok(())
    }

    // ------------------------------------------------------------------------------- viewport data for scaler
    fn calc_ratio(&self) -> [f32; 4] {
        let w = LOGIC_WIDTH as f32;
        let h = LOGIC_HEIGHT as f32;

        let surface_w = self.window.inner_size().width as f32;
        let surface_h = self.window.inner_size().height as f32;

        let ratio = h / w;
        let surface_ratio = surface_h / surface_w;

        let ratio = if ratio <= surface_ratio {
            surface_w / w
        } else {
            surface_h / h
        };

        let w = w * ratio;
        let h = h * ratio;

        let x = (surface_w - w) / 2.0;
        let y = (surface_h - h) / 2.0;

        [x, y, w, h]
    }

    // ------------------------------------------------------------------------------- window data for App
    pub fn get_window(&self) -> &Window {
        &self.window
    }

    // ------------------------------------------------------------------------------- resize surface
    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.is_surface_configured = true;
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    // ------------------------------------------------------------------------------- update uniform buffer
    pub fn update(&self, time: f32, input: f32) {
        self.queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[Uniforms { time, input }]),
        );
    }
}

// ----------------------------------------------------------------------------------- create texture to handle image
fn create_diffuse_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    label: &str,
    diffuse_bytes: &[u8],
) -> wgpu::TextureView {
    // ------------------------------------------------------------------------------- image data
    let image = image::load_from_memory(diffuse_bytes).unwrap();

    let diffuse_rgba = image.to_rgba8();
    let dimentions = image.dimensions();

    // ------------------------------------------------------------------------------- create texture
    let diffuse_texture_size = wgpu::Extent3d {
        width: dimentions.0,
        height: dimentions.1,
        depth_or_array_layers: 1,
    };

    let diffuse_texture = device.create_texture(&wgpu::wgt::TextureDescriptor {
        label: Some(label),
        size: diffuse_texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    // ------------------------------------------------------------------------------- write texture
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &diffuse_texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &diffuse_rgba,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4 * dimentions.0),
            rows_per_image: Some(dimentions.1),
        },
        diffuse_texture_size,
    );

    // ------------------------------------------------------------------------------- texture view
    diffuse_texture.create_view(&wgpu::TextureViewDescriptor::default())
}

// ----------------------------------------------------------------------------------- create texture to draw onto
fn create_texture(device: &wgpu::Device, label: &str, size: &Size) -> wgpu::TextureView {
    let new_texture_size = wgpu::Extent3d {
        width: size.width,
        height: size.height,
        depth_or_array_layers: 1,
    };

    let new_texture = device.create_texture(&wgpu::wgt::TextureDescriptor {
        label: Some(label),
        size: new_texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    // ------------------------------------------------------------------------------- texture view
    new_texture.create_view(&wgpu::TextureViewDescriptor::default())
}

// ----------------------------------------------------------------------------------- create bind group to sample/draw
fn create_simple_bind_group(
    device: &wgpu::Device,
    label: &str,
    bind_group_layout: &wgpu::BindGroupLayout,
    resource: &wgpu::TextureView,
    sampler: &wgpu::Sampler,
) -> wgpu::BindGroup {
    let new_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(label),
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&resource),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    });
    new_bind_group
}

// ----------------------------------------------------------------------------------- create bind group with uniform
fn create_effect_bind_group(
    device: &wgpu::Device,
    label: &str,
    bind_group_layout: &wgpu::BindGroupLayout,
    uniform_buffer: &wgpu::Buffer,
    texture_view: &wgpu::TextureView,
    sampler: &wgpu::Sampler,
) -> wgpu::BindGroup {
    let effect_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(label),
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: uniform_buffer,
                    offset: 0,
                    size: None,
                }),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(texture_view),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
        ],
    });
    effect_bind_group
}

// ----------------------------------------------------------------------------------- create bind group to blend
#[allow(unused)]
fn create_blend_bind_group(
    device: &wgpu::Device,
    label: &str,
    bind_group_layout: &wgpu::BindGroupLayout,
    resource_1: &wgpu::TextureView,
    resource_2: &wgpu::TextureView,
    sampler: &wgpu::Sampler,
) -> wgpu::BindGroup {
    let new_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(label),
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&resource_1),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(&resource_2),
            },
        ],
    });
    new_bind_group
}

// ----------------------------------------------------------------------------------- create render pipeline
fn create_pipeline(
    device: &wgpu::Device,
    label: &str,
    bind_group_layout: &wgpu::BindGroupLayout,
    shader: &wgpu::ShaderModule,
    blend_state: wgpu::BlendState,
) -> wgpu::RenderPipeline {
    let new_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some(label),
        bind_group_layouts: &[Some(&bind_group_layout)],
        immediate_size: 0,
    });

    let new_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(label),
        layout: Some(&new_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[Some(Vertex::desc())],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                blend: Some(blend_state),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    });
    new_render_pipeline
}

// ------------------------------------------------------------------- imports

use anyhow::Context;

use std::sync::Arc;
use winit::event_loop::ActiveEventLoop;
use winit::window::Window;

use std::borrow::Cow;
use std::mem;

use wgpu::{PipelineCompilationOptions, util::DeviceExt};

use glam::{Mat4, Vec3, camera};

use crate::config::{LOGIC_HEIGHT, LOGIC_WIDTH, RECT_HEIGHT, RECT_WIDTH};

mod bindgroup;
mod init;
mod texture;

// ------------------------------------------------------------------- struct for uniform buffer

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Uniforms {
    pub view_matrix: [[f32; 4]; 4],
    pub projection_matrix: [[f32; 4]; 4],
}

// ------------------------------------------------------------------- struct for vertex buffer

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
    uv: [f32; 2],
}

// ------------------------------------------------------------------- descriptor for VertexBufferLayout

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2];
    // 0 => Vertex::position, 1 => Vertex::uv

    const fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

// ------------------------------------------------------------------- full screen triangle

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

// ------------------------------------------------------------------- vertices to draw a rectangle

const RECT_VERTICES: &[Vertex] = &[
    Vertex {
        position: [0.0, 0.0],
        uv: [0.0, 0.0], // top left
    },
    Vertex {
        position: [0.0, RECT_HEIGHT as f32],
        uv: [0.0, 1.0], // bottom left
    },
    Vertex {
        position: [RECT_WIDTH as f32, 0.0],
        uv: [1.0, 0.0], // top right
    },
    Vertex {
        position: [RECT_WIDTH as f32, RECT_HEIGHT as f32],
        uv: [1.0, 1.0], // bottom right
    },
];

const RECT_INDICES: &[u16] = &[0, 1, 2, /**/ 1, 3, 2];

// ------------------------------------------------------------------- struct for instance buffer

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceData {
    pub position: [f32; 2], // x, y
    pub size: [f32; 2],     // w, h
    pub sprite_offset: [f32; 2],
    pub sprite_size: [f32; 2],
}

// ------------------------------------------------------------------- descriptor for VertexBufferLayout

impl InstanceData {
    const ATTRIBS: [wgpu::VertexAttribute; 4] =
        wgpu::vertex_attr_array![2 => Float32x2, 3 => Float32x2, 4 => Float32x2, 5 => Float32x2];
    // 2 => InstanceData::position, 3 => InstanceData::size, 4 => sprite_offset, 5 => sprite_size

    const fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}

// ------------------------------------------------------------------- viewport for scaler renderpass

struct ViewportData {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

// ------------------------------------------------------------------- renderer

pub struct Renderer {
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

        // ----------------------------------------------------------- load shaders

        let mid_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("mid shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("../shaders/mid.wgsl"))),
        });

        let scaler_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("scaler shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("../shaders/scaler.wgsl"))),
        });

        // ----------------------------------------------------------- uniform buffer

        let initial_uniforms = Uniforms {
            view_matrix: Mat4::IDENTITY.to_cols_array_2d(),
            projection_matrix: camera::lh::proj::directx::orthographic(
                0.0,
                f32::from(LOGIC_WIDTH),
                f32::from(LOGIC_HEIGHT),
                0.0,
                -1.0,
                1.0,
            )
            .to_cols_array_2d(),
        };

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("uniform buffer"),
            contents: bytemuck::cast_slice(&[initial_uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // ----------------------------------------------------------- full screen vertex buffer

        let fullscreen_vertex_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("full screen vertex buffer"),
                contents: bytemuck::cast_slice(FULLSCREEN_VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            });

        // ----------------------------------------------------------- rectangle vertex buffer

        let rect_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("rect vertex buffer"),
            contents: bytemuck::cast_slice(RECT_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        // ----------------------------------------------------------- instance buffer

        let instances = instances.to_vec();

        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("instance buffer"),
            contents: bytemuck::cast_slice(&instances),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        // ----------------------------------------------------------- index buffer

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("index Buffer"),
            contents: bytemuck::cast_slice(RECT_INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let num_indices = u32::try_from(RECT_INDICES.len()).context("conversion error")?;

        // ----------------------------------------------------------- samplers

        let sampler_nearest = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        // let u_bind_group_layout = bindgroup::create_u_bind_group_layout(&device);

        let t_s_bind_group_layout = bindgroup::create_t_s_bind_group_layout(&device);

        let u_t_s_bind_group_layout = bindgroup::create_u_t_s_bind_group_layout(&device);

        // let t_t_s_bind_group_layout = bindgroup::create_t_t_s_bind_group_layout(&device);

        // --------------------------------------------------------------------------- load image
        let diffuse_bytes = include_bytes!("../../img/flowers.png");

        let diffuse_texture_view =
            texture::create_diffuse_texture(&device, &queue, "diffuse texture", diffuse_bytes)?;

        // ----------------------------------------------------------- mid

        let mid_texture_view = texture::create_texture(
            &device,
            "mid texture",
            &texture::TextureSize {
                width: u32::from(LOGIC_WIDTH),
                height: u32::from(LOGIC_HEIGHT),
            },
        );

        let mid_bind_group = bindgroup::create_u_t_s_bind_group(
            &device,
            "mid bind group",
            &u_t_s_bind_group_layout,
            &uniform_buffer,
            &diffuse_texture_view,
            &sampler_nearest,
        );

        let mid_render_pipeline = create_pipeline_with_instance(
            &device,
            "render pipeline for mid",
            Some(&u_t_s_bind_group_layout),
            &mid_shader,
            wgpu::BlendState::ALPHA_BLENDING,
        );

        // ----------------------------------------------------------- scaler

        let scaler_bind_group = bindgroup::create_t_s_bind_group(
            &device,
            "scaler bind group",
            &t_s_bind_group_layout,
            &mid_texture_view,
            &sampler_nearest,
        );

        let scaler_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("render pipeline layout for scaler"),
                bind_group_layouts: &[Some(&t_s_bind_group_layout)],
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
                    compilation_options: PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &scaler_shader,
                    entry_point: Some("fs_main"),
                    compilation_options: PipelineCompilationOptions::default(),
                    targets: &[Some(swapchain_format.into())],
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview_mask: None,
                cache: None,
            });

        // ----------------------------------------------------------- renderer

        let renderer = Self {
            window,
            device,
            queue,
            surface,
            is_surface_configured: false,
            config,

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

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("render encoder"),
            });

        // ----------------------------------------------------------- mid renderpass

        let mut mid_renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.mid_texture_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.1,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        // ----------------------------------------------------------- use the renderpass

        mid_renderpass.set_pipeline(&self.mid_render_pipeline);
        mid_renderpass.set_bind_group(0, Some(&self.mid_bind_group), &[]);
        mid_renderpass.set_vertex_buffer(0, self.rect_vertex_buffer.slice(..));
        mid_renderpass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        mid_renderpass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        mid_renderpass.draw_indexed(
            0..self.num_indices,
            0,
            0..u32::try_from(self.instances.len()).context("conversion error")?,
        );

        // ----------------------------------------------------------- end the renderpass

        drop(mid_renderpass);

        // ----------------------------------------------------------- surface renderpass

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

        let viewport_data = calc_ratio(
            self.window.inner_size().width,
            self.window.inner_size().height,
        );

        // ----------------------------------------------------------- use the renderpass

        scaler_renderpass.set_pipeline(&self.scaler_render_pipeline);
        scaler_renderpass.set_bind_group(0, Some(&self.scaler_bind_group), &[]);
        scaler_renderpass.set_vertex_buffer(0, self.fullscreen_vertex_buffer.slice(..));
        scaler_renderpass.set_viewport(
            viewport_data.x,
            viewport_data.y,
            viewport_data.w,
            viewport_data.h,
            0.0,
            1.0,
        );
        scaler_renderpass.draw(0..3, 0..1);

        // ----------------------------------------------------------- end the renderpass

        drop(scaler_renderpass);

        // ----------------------------------------------------------- submit the command

        self.queue.submit([encoder.finish()]);
        self.queue.present(surface_texture);

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

// ------------------------------------------------------------------- create render pipeline

// fn create_pipeline(
//     device: &wgpu::Device,
//     label: &str,
//     bind_group_layout: Option<&wgpu::BindGroupLayout>,
//     shader: &wgpu::ShaderModule,
//     blend_state: wgpu::BlendState,
// ) -> wgpu::RenderPipeline {
//     let new_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
//         label: Some(label),
//         bind_group_layouts: &[bind_group_layout],
//         immediate_size: 0,
//     });
//
//     device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
//         label: Some(label),
//         layout: Some(&new_pipeline_layout),
//         vertex: wgpu::VertexState {
//             module: shader,
//             entry_point: Some("vs_main"),
//             buffers: &[Some(Vertex::desc())],
//             compilation_options: PipelineCompilationOptions::default(),
//         },
//         fragment: Some(wgpu::FragmentState {
//             module: shader,
//             entry_point: Some("fs_main"),
//             compilation_options: PipelineCompilationOptions::default(),
//             targets: &[Some(wgpu::ColorTargetState {
//                 format: wgpu::TextureFormat::Rgba8UnormSrgb,
//                 blend: Some(blend_state),
//                 write_mask: wgpu::ColorWrites::ALL,
//             })],
//         }),
//         primitive: wgpu::PrimitiveState::default(),
//         depth_stencil: None,
//         multisample: wgpu::MultisampleState::default(),
//         multiview_mask: None,
//         cache: None,
//     })
// }

// ------------------------------------------------------------------- create render pipeline w/ instance buffer

fn create_pipeline_with_instance(
    device: &wgpu::Device,
    label: &str,
    bind_group_layout: Option<&wgpu::BindGroupLayout>,
    shader: &wgpu::ShaderModule,
    blend_state: wgpu::BlendState,
) -> wgpu::RenderPipeline {
    let new_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some(label),
        bind_group_layouts: &[bind_group_layout],
        immediate_size: 0,
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(label),
        layout: Some(&new_pipeline_layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("vs_main"),
            buffers: &[Some(Vertex::desc()), Some(InstanceData::desc())],
            compilation_options: PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some("fs_main"),
            compilation_options: PipelineCompilationOptions::default(),
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
    })
}

// --------------------------------------------------------------- calculate viewport data for scaler

fn calc_ratio(surface_width: u32, surface_height: u32) -> ViewportData {
    let w = f32::from(LOGIC_WIDTH);
    let h = f32::from(LOGIC_HEIGHT);

    let surface_w = surface_width as f32;
    let surface_h = surface_height as f32;

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

    ViewportData { x, y, w, h }
}

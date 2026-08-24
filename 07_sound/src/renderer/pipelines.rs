// ---------------------------------------------------------------- imports

use crate::renderer::buffers::{InstanceData, Vertex};
use wgpu::PipelineCompilationOptions;

// ---------------------------------------------------------------- create render pipeline

// pub fn create_pipeline(
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

// ---------------------------------------------------------------- create render pipeline w/ instance buffer

pub fn create_pipeline_with_instance(
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

pub fn create_surface_pipeline(
    adapter: &wgpu::Adapter,
    device: &wgpu::Device,
    surface: &wgpu::Surface,
    bind_group_layout: &wgpu::BindGroupLayout,
    shader: &wgpu::ShaderModule,
) -> wgpu::RenderPipeline {
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("render pipeline layout for surface"),
        bind_group_layouts: &[Some(bind_group_layout)],
        immediate_size: 0,
    });

    let swapchain_capabilities = surface.get_capabilities(adapter);
    let swapchain_format = swapchain_capabilities.formats[0];

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("render pipeline for surface"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("vs_main"),
            buffers: &[Some(Vertex::desc())],
            compilation_options: PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some("fs_main"),
            compilation_options: PipelineCompilationOptions::default(),
            targets: &[Some(swapchain_format.into())],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    })
}

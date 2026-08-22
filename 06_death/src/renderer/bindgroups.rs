// ---------------------------------------------------------------- imports

use crate::renderer::buffers::Uniforms;

// ---------------------------------------------------------------- functions to create bind group layout

// pub fn create_u_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
//     // ----------------------------------------------------------- bind group w/ uniform
//
//     device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
//         label: Some("bind group layout with uniform"),
//         entries: &[wgpu::BindGroupLayoutEntry {
//             binding: 0,
//             visibility: wgpu::ShaderStages::VERTEX,
//             ty: wgpu::BindingType::Buffer {
//                 ty: wgpu::BufferBindingType::Uniform,
//                 has_dynamic_offset: false,
//                 min_binding_size: wgpu::BufferSize::new(std::mem::size_of::<Uniforms>() as u64),
//             },
//             count: None,
//         }],
//     })
// }

pub fn create_t_s_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    // ----------------------------------------------------------- bind group w/ texture and sampler

    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("bind group layout with texture and sampler"),
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
    })
}

pub fn create_u_t_s_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    // ----------------------------------------------------------- bind group w/ uniform, texture, sampler

    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("bind group layout with uniform, texture, and sampler"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(std::mem::size_of::<Uniforms>() as u64),
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
    })
}

// pub fn create_t_t_s_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
//     // ----------------------------------------------------------- bind group w/ 2 textures, 1 sampler
//
//     device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
//         label: Some("bind group layout with 2 textures, 1 sampler"),
//         entries: &[
//             wgpu::BindGroupLayoutEntry {
//                 binding: 0,
//                 visibility: wgpu::ShaderStages::FRAGMENT,
//                 ty: wgpu::BindingType::Texture {
//                     sample_type: wgpu::TextureSampleType::Float { filterable: true },
//                     view_dimension: wgpu::TextureViewDimension::D2,
//                     multisampled: false,
//                 },
//                 count: None,
//             },
//             wgpu::BindGroupLayoutEntry {
//                 binding: 1,
//                 visibility: wgpu::ShaderStages::FRAGMENT,
//                 ty: wgpu::BindingType::Texture {
//                     sample_type: wgpu::TextureSampleType::Float { filterable: true },
//                     view_dimension: wgpu::TextureViewDimension::D2,
//                     multisampled: false,
//                 },
//                 count: None,
//             },
//             wgpu::BindGroupLayoutEntry {
//                 binding: 2,
//                 visibility: wgpu::ShaderStages::FRAGMENT,
//                 ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
//                 count: None,
//             },
//         ],
//     })
// }

// ---------------------------------------------------------------- functions to create bind groups

// ---------------------------------------------------------------- create bind group w/ uniform

// pub fn create_u_bind_group(
//     device: &wgpu::Device,
//     label: &str,
//     bind_group_layout: &wgpu::BindGroupLayout,
//     uniform_buffer: &wgpu::Buffer,
// ) -> wgpu::BindGroup {
//     device.create_bind_group(&wgpu::BindGroupDescriptor {
//         label: Some(label),
//         layout: bind_group_layout,
//         entries: &[wgpu::BindGroupEntry {
//             binding: 0,
//             resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
//                 buffer: uniform_buffer,
//                 offset: 0,
//                 size: None,
//             }),
//         }],
//     })
// }

// ---------------------------------------------------------------- create bind group w/ texture and sampler

pub fn create_t_s_bind_group(
    device: &wgpu::Device,
    label: &str,
    bind_group_layout: &wgpu::BindGroupLayout,
    resource: &wgpu::TextureView,
    sampler: &wgpu::Sampler,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(label),
        layout: bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(resource),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
        ],
    })
}

// ---------------------------------------------------------------- create bind group w/ uniform, texture, sampler

pub fn create_u_t_s_bind_group(
    device: &wgpu::Device,
    label: &str,
    bind_group_layout: &wgpu::BindGroupLayout,
    uniform_buffer: &wgpu::Buffer,
    texture_view: &wgpu::TextureView,
    sampler: &wgpu::Sampler,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(label),
        layout: bind_group_layout,
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
    })
}

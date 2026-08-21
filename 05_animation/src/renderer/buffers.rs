use std::mem;
use wgpu::util::DeviceExt;

use crate::config::{LOGIC_HEIGHT, LOGIC_WIDTH, RECT_HEIGHT, RECT_WIDTH};
use glam::{Mat4, Vec3, camera};

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
pub struct Vertex {
    position: [f32; 2],
    uv: [f32; 2],
}

// ------------------------------------------------------------------- descriptor for VertexBufferLayout

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2];
    // 0 => Vertex::position, 1 => Vertex::uv

    pub const fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

// ------------------------------------------------------------------- full screen triangle

pub const FULLSCREEN_VERTICES: &[Vertex] = &[
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

pub const RECT_VERTICES: &[Vertex] = &[
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

pub const RECT_INDICES: &[u16] = &[0, 1, 2, /**/ 1, 3, 2];

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

    pub const fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}

// ------------------------------------------------------------------- functions to create buffer

pub fn create_uniform_buffer(device: &wgpu::Device) -> wgpu::Buffer {
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

    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("uniform buffer"),
        contents: bytemuck::cast_slice(&[initial_uniforms]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    })
}

// --------------------------------------------------------------- full screen vertex buffer

pub fn create_vertex_buffer(
    device: &wgpu::Device,
    label: &str,
    vertices: &[Vertex],
) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents: bytemuck::cast_slice(vertices),
        usage: wgpu::BufferUsages::VERTEX,
    })
}

// --------------------------------------------------------------- instance buffer

pub fn create_instance_buffer(device: &wgpu::Device, instances: &[InstanceData]) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("instance buffer"),
        contents: bytemuck::cast_slice(instances),
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
    })
}

// --------------------------------------------------------------- index buffer

pub fn create_index_buffer(device: &wgpu::Device, indices: &[u16]) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("index Buffer"),
        contents: bytemuck::cast_slice(indices),
        usage: wgpu::BufferUsages::INDEX,
    })
}

pub fn update_uniform_buffer(queue: &wgpu::Queue, uniform_buffer: &mut wgpu::Buffer) {
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

    queue.write_buffer(
        uniform_buffer,
        0,
        bytemuck::cast_slice(&[Uniforms {
            view_matrix: view.to_cols_array_2d(),
            projection_matrix: projection.to_cols_array_2d(),
        }]),
    );
}

pub fn update_instance_buffer(
    queue: &wgpu::Queue,
    instance_buffer: &mut wgpu::Buffer,
    instances: &[InstanceData],
) {
    queue.write_buffer(instance_buffer, 0, bytemuck::cast_slice(instances));
}

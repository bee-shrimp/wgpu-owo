// ---------------------------------------------------------------- imports

use color_eyre::eyre::Result;
use winit::dpi::PhysicalSize;

use crate::{
    config::{LOGIC_HEIGHT, LOGIC_WIDTH},
    renderer::buffers,
};

// ---------------------------------------------------------------- viewport for scaler renderpass

struct ViewportData {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

/// draw instances onto a small texture
#[allow(clippy::too_many_arguments)]
pub fn draw_mid_renderpass(
    encoder: &mut wgpu::CommandEncoder,

    mid_texture_view: &wgpu::TextureView,
    mid_render_pipeline: &wgpu::RenderPipeline,
    mid_bind_group: &wgpu::BindGroup,

    rect_vertex_buffer: &wgpu::Buffer,
    instance_buffer: &wgpu::Buffer,
    index_buffer: &wgpu::Buffer,

    num_instances: u32,
) -> Result<()> {
    // ------------------------------------------------------------ mid renderpass

    let mut mid_renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: None,
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: mid_texture_view,
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

    let num_indices = u32::try_from(buffers::RECT_INDICES.len())?;

    // ------------------------------------------------------------ use the renderpass

    mid_renderpass.set_pipeline(mid_render_pipeline);
    mid_renderpass.set_bind_group(0, Some(mid_bind_group), &[]);
    mid_renderpass.set_vertex_buffer(0, rect_vertex_buffer.slice(..));
    mid_renderpass.set_vertex_buffer(1, instance_buffer.slice(..));
    mid_renderpass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
    mid_renderpass.draw_indexed(0..num_indices, 0, 0..num_instances);

    // ------------------------------------------------------------ end the renderpass

    drop(mid_renderpass);

    Ok(())
}

/// samples a texture and draw bigger onto the surface.
pub fn draw_scaler_renderpass(
    encoder: &mut wgpu::CommandEncoder,

    surface_texture_view: &wgpu::TextureView,
    scaler_render_pipeline: &wgpu::RenderPipeline,
    scaler_bind_group: &wgpu::BindGroup,

    fullscreen_vertex_buffer: &wgpu::Buffer,

    window_size: PhysicalSize<u32>,
) {
    // ------------------------------------------------------------ surface renderpass

    let mut scaler_renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("scaler renderpass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: surface_texture_view,
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

    // u32->f32 as conversion allowance for ratio calculation
    #[allow(clippy::as_conversions, clippy::cast_precision_loss)]
    let viewport_data = calc_ratio(window_size.width as f32, window_size.height as f32);

    // ------------------------------------------------------------ use the renderpass

    scaler_renderpass.set_pipeline(scaler_render_pipeline);
    scaler_renderpass.set_bind_group(0, Some(scaler_bind_group), &[]);
    scaler_renderpass.set_vertex_buffer(0, fullscreen_vertex_buffer.slice(..));
    scaler_renderpass.set_viewport(
        viewport_data.x,
        viewport_data.y,
        viewport_data.w,
        viewport_data.h,
        0.0,
        1.0,
    );
    scaler_renderpass.draw(0..3, 0..1);

    // ------------------------------------------------------------ end the renderpass

    drop(scaler_renderpass);
}

/// calculates aspect ratio and returns viewport data.
fn calc_ratio(surface_w: f32, surface_h: f32) -> ViewportData {
    let w = f32::from(LOGIC_WIDTH);
    let h = f32::from(LOGIC_HEIGHT);

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

// ---------------------------------------------------------------- imports

use color_eyre::eyre::{Result, WrapErr};

use image::GenericImageView;

// ---------------------------------------------------------------- texture size for create_texture()

pub struct TextureSize {
    pub width: u32,
    pub height: u32,
}

// ---------------------------------------------------------------- functions to create texture.

/// returns texture with image data written.
pub fn create_diffuse_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    label: &str,
    diffuse_bytes: &[u8],
) -> Result<wgpu::TextureView> {
    // ------------------------------------------------------------ image data

    let image = image::load_from_memory(diffuse_bytes).wrap_err("failed to load image")?;

    let diffuse_rgba = image.to_rgba8();
    let dimensions = image.dimensions();

    // ------------------------------------------------------------ create texture

    let diffuse_texture_size = wgpu::Extent3d {
        width: dimensions.0,
        height: dimensions.1,
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

    // ------------------------------------------------------------ write texture

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
            bytes_per_row: Some(4 * dimensions.0),
            rows_per_image: Some(dimensions.1),
        },
        diffuse_texture_size,
    );

    // ------------------------------------------------------------ texture view

    Ok(diffuse_texture.create_view(&wgpu::TextureViewDescriptor::default()))
}

/// returns texture view to draw onto / to be read.
pub fn create_texture(device: &wgpu::Device, label: &str, size: &TextureSize) -> wgpu::TextureView {
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

    // ------------------------------------------------------------ texture view

    new_texture.create_view(&wgpu::TextureViewDescriptor::default())
}

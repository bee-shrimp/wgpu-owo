// ---------------------------------------------------------------- imports

use anyhow::{Context, Result};
use std::sync::Arc;
use winit::event_loop::ActiveEventLoop;
use winit::window::Window;

use wgpu::{BackendOptions, InstanceFlags, MemoryBudgetThresholds};

pub async fn init_wgpu(
    window: &Arc<Window>,
    event_loop: &ActiveEventLoop,
) -> Result<(
    wgpu::Adapter,
    wgpu::Device,
    wgpu::Queue,
    wgpu::Surface<'static>,
    wgpu::SurfaceConfiguration,
)> {
    // ------------------------------------------------------------ size of window

    let size = window.inner_size();

    // ------------------------------------------------------------ create a new wgpu instance

    let wgpu_instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::GL,
        flags: InstanceFlags::default(),
        memory_budget_thresholds: MemoryBudgetThresholds::default(),
        backend_options: BackendOptions::default(),
        display: Some(Box::new(event_loop.owned_display_handle())),
    });

    // ------------------------------------------------------------ physical device

    let adapter = wgpu_instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: None,
            force_fallback_adapter: false,
            apply_limit_buckets: true,
        })
        .await
        .context("failed to request adapter")?;

    // ------------------------------------------------------------ logical device

    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor::default())
        .await
        .context("failed to create device")?;

    // ------------------------------------------------------------ surface to draw onto

    let surface = wgpu_instance
        .create_surface(window.clone())
        .context("failed to create surface")?;

    let surface_caps = surface.get_capabilities(&adapter);

    let surface_format = surface_caps
        .formats
        .iter()
        .copied()
        .find(wgpu::TextureFormat::is_srgb)
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
    Ok((adapter, device, queue, surface, config))
}

pub fn create_sampler(
    device: &wgpu::Device,
    filter_mode: wgpu::FilterMode,
    mipmap_filter_mode: wgpu::MipmapFilterMode,
) -> wgpu::Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: filter_mode,
        min_filter: filter_mode,
        mipmap_filter: mipmap_filter_mode,
        ..Default::default()
    })
}

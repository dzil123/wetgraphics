use wgpu::{
    AddressMode, BindGroup, BindGroupLayout, Extent3d, FilterMode, Instance, Sampler,
    SamplerDescriptor, Surface, SwapChainDescriptor, Texture, TextureDescriptor, TextureDimension,
    TextureFormat, TextureUsage, TextureView, TextureViewDimension,
};

use crate::wgpu::WgpuBase;

pub trait SafeWgpuSurface {
    fn create_surface(&self, instance: &Instance) -> Surface;
}

pub type WindowSize = winit::dpi::PhysicalSize<u32>;

pub trait CreateFromWgpu {
    fn new(wgpu_base: &WgpuBase, desc: &TextureDesc) -> Self;
}

#[derive(Clone)]
pub struct TextureDesc {
    pub width: u32,
    pub height: u32,
    pub format: TextureFormat,
}

impl TextureDesc {
    pub fn into_2d(&self, usage: TextureUsage) -> TextureDescriptor<'_> {
        TextureDescriptor {
            label: None,
            size: Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: self.format,
            usage,
        }
    }

    pub fn into_2d_array(&self, usage: TextureUsage, length: u32) -> TextureDescriptor<'_> {
        let base = self.into_2d(usage);

        TextureDescriptor {
            size: Extent3d {
                depth_or_array_layers: length,
                ..base.size
            },
            ..base
        }
    }

    pub fn into_3d(&self, usage: TextureUsage, depth: u32) -> TextureDescriptor<'_> {
        let base = self.into_2d_array(usage, depth);

        TextureDescriptor {
            dimension: TextureDimension::D3,
            ..base
        }
    }
}

impl From<&SwapChainDescriptor> for TextureDesc {
    fn from(desc: &SwapChainDescriptor) -> Self {
        Self {
            width: desc.width,
            height: desc.height,
            format: desc.format,
        }
    }
}

#[derive(Default)]
pub struct SamplerDesc {
    filter: bool,
    address: AddressMode,
}

impl From<SamplerDesc> for SamplerDescriptor<'static> {
    fn from(desc: SamplerDesc) -> Self {
        let filter = if desc.filter {
            FilterMode::Linear
        } else {
            FilterMode::Nearest
        };

        SamplerDescriptor {
            address_mode_u: desc.address,
            address_mode_v: desc.address,
            address_mode_w: desc.address,
            mag_filter: filter,
            min_filter: filter,
            mipmap_filter: filter,
            ..Default::default()
        }
    }
}

pub fn texture_view_dimension(desc: &TextureDescriptor<'_>) -> TextureViewDimension {
    // https://github.com/gfx-rs/wgpu/blob/7a50f12cd4/wgpu-core/src/device/mod.rs#L794-L801
    match (desc.dimension, desc.size.depth_or_array_layers) {
        (TextureDimension::D1, _) => TextureViewDimension::D1,
        (TextureDimension::D2, depth) if depth > 1 => TextureViewDimension::D2Array,
        (TextureDimension::D2, _) => TextureViewDimension::D2,
        (TextureDimension::D3, _) => TextureViewDimension::D3,
    }
}

// wgpu does implicit Arc<> semantics in the background, so eg everything can be dropped but BindGroup will remain valid
pub struct TextureResult {
    pub texture: Texture,
    pub view: TextureView,
    pub sampler: Sampler,
    pub bind_layout: BindGroupLayout,
    pub bind: BindGroup,
}

pub fn texture_size(desc: &TextureDescriptor<'_>) -> usize {
    let size = desc.size;
    let fmt = desc.format.describe();

    if fmt.block_dimensions != (1, 1) || desc.mip_level_count != 1 {
        unimplemented!(); // ignore compressed formats
    }

    (fmt.block_size as usize)
        * (size.width as usize)
        * (size.height as usize)
        * (size.depth_or_array_layers as usize)
}

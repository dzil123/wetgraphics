use std::num::NonZeroU32;

use wgpu::{
    AddressMode, Extent3d, FilterMode, ImageDataLayout, Instance, SamplerDescriptor, Surface,
    SwapChainDescriptor, TextureDescriptor, TextureDimension, TextureFormat, TextureUsage,
    TextureViewDimension, COPY_BYTES_PER_ROW_ALIGNMENT,
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
    pub fn into_2d(&self, usage: TextureUsage) -> TextureDescriptor<'static> {
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
            usage, // todo: replace with self.format.describe().guaranteed_format_features.allowed_usages?
        }
    }

    pub fn into_2d_array(&self, usage: TextureUsage, length: u32) -> TextureDescriptor<'static> {
        let base = self.into_2d(usage);

        TextureDescriptor {
            size: Extent3d {
                depth_or_array_layers: length,
                ..base.size
            },
            ..base
        }
    }

    pub fn into_3d(&self, usage: TextureUsage, depth: u32) -> TextureDescriptor<'static> {
        let base = self.into_2d_array(usage, depth);

        TextureDescriptor {
            dimension: TextureDimension::D3,
            ..base
        }
    }

    pub fn aligned(&self) -> Self {
        let pixel_align = COPY_BYTES_PER_ROW_ALIGNMENT / (self.format.describe().block_size as u32);

        Self {
            width: (((self.width - 1) / pixel_align) + 1) * pixel_align,
            ..self.clone()
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

#[derive(Copy, Clone, Default)]
pub struct SamplerDesc {
    pub filter: bool,
    pub address: AddressMode,
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

pub fn texture_image_layout(desc: &TextureDescriptor<'_>) -> ImageDataLayout {
    let size = desc.size;

    ImageDataLayout {
        bytes_per_row: if size.height > 1 {
            NonZeroU32::new(size.width * (desc.format.describe().block_size as u32))
        } else {
            None
        },
        rows_per_image: if size.depth_or_array_layers > 1 {
            NonZeroU32::new(size.height)
        } else {
            None
        },
        ..Default::default()
    }
}

pub fn to_image(data: &[u8], desc: &TextureDesc) -> image::ImageResult<image::DynamicImage> {
    struct Decoder<'a> {
        data: &'a [u8],
        desc: &'a TextureDesc,
    }

    impl<'a> image::ImageDecoder<'a> for Decoder<'a> {
        type Reader = &'a [u8];

        fn dimensions(&self) -> (u32, u32) {
            (self.desc.width, self.desc.height)
        }

        fn color_type(&self) -> image::ColorType {
            match self.desc.format {
                TextureFormat::Bgra8Unorm => image::ColorType::Bgra8,
                _ => unimplemented!(),
            }
        }

        fn into_reader(self) -> image::ImageResult<Self::Reader> {
            Ok(self.data)
        }
    }

    image::DynamicImage::from_decoder(Decoder { data, desc })
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

pub enum InitType<'a> {
    Uninit,             // simple create
    Zeros,              // allocate a vec of 0s and do init create
    Repeated(&'a [u8]), // eg texture color, init create
    Data(&'a [u8]),     // init create
}

impl<'a> InitType<'a> {
    pub fn create<T>(
        self,
        size: usize,
        uninit: impl FnOnce() -> T,
        init: impl FnOnce(&[u8]) -> T,
    ) -> T {
        let mut vec = Vec::new();

        let data: &[u8] = match self {
            Self::Uninit => return uninit(),
            Self::Zeros => {
                vec.resize(size, 0);
                &vec
            }
            Self::Repeated(word) => {
                vec = std::iter::repeat(word)
                    .flatten()
                    .copied()
                    .take(size)
                    .collect();
                &vec
            }
            Self::Data(data) => data,
        };

        init(data)
    }
}

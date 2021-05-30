use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, Buffer, BufferBindingType, ShaderStage,
    StorageTextureAccess, TextureDescriptor, TextureView,
};

use crate::util::{texture_view_dimension, SamplerDesc};

use super::WgpuBase;

impl WgpuBase {
    pub fn bind_group(&self, entries: &[BindGroupEntry<'_>]) -> BindGroupResult {
        let layout_entries: Vec<_> = entries
            .iter()
            .enumerate()
            .map(|(index, entry)| BindGroupLayoutEntry {
                binding: index as _,
                visibility: ShaderStage::all(),
                ty: entry.as_layout(),
                count: None,
            })
            .collect();

        let layout = self
            .device
            .create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: None,
                entries: &layout_entries,
            });

        let bind_entries: Vec<_> = entries
            .iter()
            .enumerate()
            .map(|(index, entry)| wgpu::BindGroupEntry {
                binding: index as _,
                resource: entry.as_bind(),
            })
            .collect();

        let bind = self.device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &layout,
            entries: &bind_entries,
        });

        BindGroupResult { layout, bind }
    }
}

pub struct BindGroupResult {
    pub layout: BindGroupLayout,
    pub bind: BindGroup,
}

pub enum BindGroupEntry<'a> {
    Buffer {
        ty: BufferBindingType,
        buffer: &'a Buffer,
    },
    Sampler {
        desc: SamplerDesc,
    },
    Texture {
        // texture or storagetexture
        storage: Option<StorageTextureAccess>,
        desc: TextureDescriptor<'static>,
        view: &'a TextureView,
    },
}

impl<'a> BindGroupEntry<'a> {
    fn as_layout(&self) -> BindingType {
        match self {
            Self::Buffer { ty, .. } => BindingType::Buffer {
                ty: *ty,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            Self::Sampler {
                desc: SamplerDesc { filter, .. },
            } => BindingType::Sampler {
                filtering: *filter,
                comparison: false,
            },
            Self::Texture {
                storage: None,
                desc,
                view,
            } => BindingType::Texture {
                sample_type: desc.format.describe().sample_type,
                view_dimension: texture_view_dimension(desc),
                multisampled: false,
            },
            Self::Texture {
                storage: Some(access),
                desc,
                view,
            } => BindingType::StorageTexture {
                access: *access,
                format: desc.format,
                view_dimension: texture_view_dimension(desc),
            },
        }
    }

    fn as_bind<'b>(&self) -> BindingResource<'b>
    where
        'a: 'b,
    {
        match self {
            Self::Buffer { buffer, .. } => buffer.as_entire_binding(),
            Self::Sampler { desc } => {
                let sampler = todo!();
                BindingResource::Sampler(sampler)
            }
            Self::Texture { view, .. } => BindingResource::TextureView(view),
        }
    }
}

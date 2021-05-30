use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, BufferDescriptor, BufferUsage,
};

use crate::util::InitType;

use super::WgpuBase;

impl WgpuBase {
    pub fn buffer(&self, desc: BufferDesc, init: InitType<'_>) -> Buffer {
        init.create(
            desc.size,
            || {
                self.device.create_buffer(&BufferDescriptor {
                    label: None,
                    size: desc.size as _,
                    usage: desc.usage,
                    mapped_at_creation: false,
                })
            },
            |data| {
                self.device.create_buffer_init(&BufferInitDescriptor {
                    label: None,
                    contents: data,
                    usage: desc.usage,
                })
            },
        )
    }
}

pub struct BufferDesc {
    pub size: usize, // todo: instead take a std140 type and compute size from that?
    pub usage: BufferUsage,
}

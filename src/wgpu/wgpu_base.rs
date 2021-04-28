use std::iter;

use pollster::FutureExt as _;
use wgpu::{
    Adapter, BackendBit, Color, CommandEncoder, Device, Features, Instance, Limits, LoadOp,
    Operations, PowerPreference, Queue, RenderPass, RenderPassColorAttachmentDescriptor,
    RenderPassDescriptor, RequestAdapterOptions, ShaderModule, Surface, TextureView,
};

use crate::util::SafeWgpuSurface;

// simple render pass that only clears the frame to black. ignore if using depth buffer, not clearing frame, or anything more complex
fn begin_render_pass<'a>(
    encoder: &'a mut CommandEncoder,
    texture: &'a TextureView,
) -> RenderPass<'a> {
    encoder.begin_render_pass(&RenderPassDescriptor {
        color_attachments: &[RenderPassColorAttachmentDescriptor {
            attachment: texture,
            resolve_target: None,
            ops: Operations {
                load: LoadOp::Clear(Color::BLACK),
                ..Default::default()
            },
        }],
        ..Default::default()
    })
}

fn create_instance() -> Instance {
    Instance::new(BackendBit::VULKAN)
}

fn create_adapter(instance: Instance, surface: Option<&Surface>) -> (Instance, Adapter) {
    let adapter = instance
        .request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::HighPerformance,
            compatible_surface: surface,
            ..Default::default()
        })
        .block_on()
        .unwrap();
    dbg!(adapter.get_info());
    (instance, adapter)
}

pub struct WgpuBase {
    pub instance: Instance,
    pub adapter: Adapter,
    pub device: Device,
    pub queue: Queue,
}

impl WgpuBase {
    fn new_impl((instance, adapter): (Instance, Adapter)) -> Self {
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: Features::PUSH_CONSTANTS,
                    limits: Limits {
                        max_push_constant_size: 128, // i have it on good authority that this is the max for a rx 580 and igpu
                        ..Default::default()
                    },
                    ..Default::default()
                },
                None,
            )
            .block_on()
            .unwrap();

        Self {
            instance,
            adapter,
            device,
            queue,
        }
    }

    pub fn new() -> Self {
        Self::new_impl(create_adapter(create_instance(), None))
    }

    pub fn new_surface<W>(window: &W) -> (Self, Surface)
    where
        W: SafeWgpuSurface,
    {
        let instance = create_instance();
        let surface = window.create_surface(&instance);
        let this = Self::new_impl(create_adapter(instance, Some(&surface)));
        (this, surface)
    }

    pub fn render<T>(&self, texture: &TextureView, target: &mut T)
    where
        T: WgpuBaseRender,
    {
        let mut encoder = self.device.create_command_encoder(&Default::default());

        {
            let mut render_pass = begin_render_pass(&mut encoder, texture);
            target.render(self, &mut render_pass);
        }

        self.queue.submit(iter::once(encoder.finish()));
    }

    pub fn shader(&self, name: &str) -> ShaderModule {
        crate::shaders::load(&self.device, name)
    }
}

// for<'b> https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=337720452d4fa161323fb2939ee23af1

pub trait WgpuBaseRender {
    fn render<'a>(&'a mut self, wgpu_base: &WgpuBase, render_pass: &mut RenderPass<'a>);
}

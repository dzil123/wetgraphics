use std::iter;

use pollster::FutureExt as _;
use wgpu::{
    Adapter, BackendBit, CommandEncoder, Device, DeviceDescriptor, Features, Instance, Limits,
    PowerPreference, Queue, RenderPass, RenderPassColorAttachment, RenderPassDescriptor,
    RequestAdapterOptions, Surface, TextureView,
};

use crate::util::SafeWgpuSurface;

// simple render pass that only clears the frame to black. ignore if using depth buffer, not clearing frame, or anything more complex
fn begin_render_pass<'a>(
    encoder: &'a mut CommandEncoder,
    texture: &'a TextureView,
) -> RenderPass<'a> {
    encoder.begin_render_pass(&RenderPassDescriptor {
        color_attachments: &[RenderPassColorAttachment {
            view: texture,
            resolve_target: None,
            ops: Default::default(),
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
    (instance, adapter)
}

pub struct WgpuBase {
    pub instance: Instance,
    pub adapter: Adapter,
    pub device: Device,
    pub queue: Queue,
    pub(super) shaders: super::shaders::Shaders,
}

impl WgpuBase {
    fn new_impl((instance, adapter): (Instance, Adapter)) -> Self {
        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    features: Features::PUSH_CONSTANTS,
                    limits: Limits {
                        max_push_constant_size: 128,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                None,
            )
            .block_on()
            .unwrap();

        dbg!(device.features());
        // device.on_uncaptured_error(|err| eprintln!("{:#?}", err));

        Self {
            instance,
            adapter,
            device,
            queue,
            shaders: Default::default(),
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
        target.render_encoder(self, &mut encoder, false);

        {
            let mut render_pass = begin_render_pass(&mut encoder, texture);
            target.render(self, &mut render_pass);
        }

        target.render_encoder(self, &mut encoder, true);
        self.queue.submit(iter::once(encoder.finish()));
    }
}

// for<'b> https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=337720452d4fa161323fb2939ee23af1

pub trait WgpuBaseRender {
    fn render<'a>(&'a mut self, wgpu_base: &WgpuBase, render_pass: &mut RenderPass<'a>);
    fn render_encoder(&mut self, wgpu_base: &WgpuBase, encoder: &mut CommandEncoder, after: bool);
}

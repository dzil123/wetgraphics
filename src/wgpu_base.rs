use std::iter;

use pollster::FutureExt as _;
use wgpu::{
    Adapter, BackendBit, Color, CommandEncoder, Device, Features, Instance, Limits, LoadOp,
    Operations, PowerPreference, Queue, RenderPass, RenderPassColorAttachmentDescriptor,
    RenderPassDescriptor, RequestAdapterOptions, Surface, TextureView,
};

use crate::{mainloop::Mainloop, util::SafeWgpuSurface};

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
    (instance, adapter)
}

struct WgpuBase<T> {
    inner: T,
    adapter: Adapter,
    device: Device,
    queue: Queue,
}

impl<T> WgpuBase<T> {
    fn new_impl(inner: T, (instance, adapter): (Instance, Adapter)) -> Self {
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
            inner,
            adapter,
            device,
            queue,
        }
    }

    pub fn new(inner: T) -> Self {
        Self::new_impl(inner, create_adapter(create_instance(), None))
    }

    pub fn new_surface<W>(inner: T, window: &W) -> (Self, Surface)
    where
        W: SafeWgpuSurface,
    {
        let instance = create_instance();
        let surface = window.create_surface(&instance);
        let this = Self::new_impl(inner, create_adapter(instance, Some(&surface)));
        (this, surface)
    }

    // fn render<T>(&self, texture: &wgpu::TextureView, mut f: T)
    // where
    //     T: FnMut(wgpu::RenderPass),
    // {
    //     let mut encoder = self.device.create_command_encoder(&Default::default());

    //     f(begin_render_pass(&mut encoder, texture));

    //     self.queue.submit(iter::once(encoder.finish()));
    // }
}

impl<'a, T> Mainloop for WgpuBase<T>
where
    T: Mainloop<RenderParams = RenderPass<'a>>,
    // 'a: 'b,
{
    type Inner = T;
    type RenderParams = &'static TextureView;

    fn render(&mut self, texture: Self::RenderParams) {
        let mut encoder = self.device.create_command_encoder(&Default::default());

        let render_pass = begin_render_pass(&mut encoder, texture);
        self.inner.render(render_pass);

        self.queue.submit(iter::once(encoder.finish()));
    }
}

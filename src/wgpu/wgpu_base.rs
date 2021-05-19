use std::iter;

use pollster::FutureExt as _;
use wgpu::{
    util::DeviceExt, Adapter, BackendBit, BindGroupDescriptor, BindGroupEntry,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, CommandEncoder,
    Device, Features, Instance, Limits, PowerPreference, Queue, RenderPass,
    RenderPassColorAttachment, RenderPassDescriptor, RequestAdapterOptions, ShaderModule,
    ShaderStage, Surface, TextureDescriptor, TextureView,
};

use crate::util::{
    texture_size, texture_view_dimension, SafeWgpuSurface, SamplerDesc, TextureResult,
};

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

        // device.on_uncaptured_error(|err| eprintln!("{:#?}", err));

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
        target.render2(self, &mut encoder);

        {
            let mut render_pass = begin_render_pass(&mut encoder, texture);
            target.render(self, &mut render_pass);
        }

        self.queue.submit(iter::once(encoder.finish()));
    }

    pub fn shader(&self, name: &str) -> ShaderModule {
        crate::shaders::load(&self.device, name)
    }

    pub fn texture(
        &self,
        desc: &TextureDescriptor<'_>,
        sampler: SamplerDesc,
        data: Option<&[u8]>,
    ) -> TextureResult {
        let mut vec = Vec::new();

        let data = data.unwrap_or_else(|| {
            vec.resize(texture_size(desc), 0);
            &vec
        });

        let texture = self
            .device
            .create_texture_with_data(&self.queue, desc, data);

        let view = texture.create_view(&Default::default());
        let sampler = self.device.create_sampler(&sampler.into()); // todo: reuse samplers, bind group layouts?

        let bind_layout = self
            .device
            .create_bind_group_layout(&BindGroupLayoutDescriptor {
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStage::all(),
                        ty: BindingType::Texture {
                            multisampled: false,
                            view_dimension: texture_view_dimension(&desc),
                            sample_type: desc.format.describe().sample_type, // it seems to not work with anything else, even though vulkan says any type is 'compatible' https://www.khronos.org/registry/vulkan/specs/1.2/html/chap33.html#formats-compatibility-classes
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStage::all(),
                        ty: BindingType::Sampler {
                            comparison: false,
                            filtering: true,
                        },
                        count: None,
                    },
                ],
                label: None,
            });

        let bind = self.device.create_bind_group(&BindGroupDescriptor {
            layout: &bind_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&sampler),
                },
            ],
            label: None,
        });

        TextureResult {
            texture,
            view,
            sampler,
            bind_layout,
            bind,
        }
    }
}

// for<'b> https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=337720452d4fa161323fb2939ee23af1

pub trait WgpuBaseRender {
    fn render<'a>(&'a mut self, wgpu_base: &WgpuBase, render_pass: &mut RenderPass<'a>);
    fn render2(&mut self, wgpu_base: &WgpuBase, encoder: &mut CommandEncoder) {}
}

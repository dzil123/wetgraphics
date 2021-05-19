use bytemuck::bytes_of;
use wgpu::{
    util::DeviceExt, BindGroup, BindGroupEntry, BindGroupLayoutEntry, BindingResource, BindingType,
    BlendState, ColorTargetState, ColorWrite, Face, FilterMode, FragmentState,
    PipelineLayoutDescriptor, PrimitiveState, PushConstantRange, RenderPass, RenderPipeline,
    RenderPipelineDescriptor, Sampler, SamplerDescriptor, ShaderStage, Texture, TextureDescriptor,
    TextureDimension, TextureFormat, TextureSampleType, TextureUsage, TextureView,
    TextureViewDimension, VertexState,
};

use crate::util::{texture_view_dimension, CreateFromWgpu, TextureResult};
use crate::wgpu::{WgpuBase, WgpuWindowed, WgpuWindowedRender};
use crate::{imgui::ImguiWgpuRender, util::TextureDesc};

const COLORS: [[f32; 3]; 4] = [
    [0.0, 0.0, 0.0],
    [1.0, 0.0, 0.0],
    [0.0, 1.0, 0.0],
    [0.0, 0.0, 1.0],
];

pub struct App {
    render_pipeline: RenderPipeline,
    color_index: usize,
    // texture: Texture,
    // texture_view: TextureView,
    // texture_sampler: Sampler,
    texture_bind_group: BindGroup,
}

impl CreateFromWgpu for App {
    fn new(wgpu_base: &WgpuBase, desc: &TextureDesc) -> Self {
        let device = &wgpu_base.device;

        let num_bytes = 4 * desc.width * desc.height;
        let data = vec![0; num_bytes as usize];

        let data: Vec<u8> = (0..num_bytes).into_iter().map(|x| (x * 85) as _).collect();

        let data2: Vec<u8> = std::iter::repeat([255, 0, 0, 255].iter())
            .flatten()
            .map(|&x| x)
            .take(num_bytes as _)
            .collect();

        let desc = desc.into_2d(TextureUsage::COPY_DST | TextureUsage::SAMPLED);
        let TextureResult {
            bind_layout, bind, ..
        } = wgpu_base.texture(&desc, Default::default(), Some(&data2));

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&device.create_pipeline_layout(&PipelineLayoutDescriptor {
                bind_group_layouts: &[&bind_layout],
                // push_constant_ranges: &[PushConstantRange {
                //     stages: ShaderStage::all(),
                //     range: 0..(4 * 3),
                // }],
                ..Default::default()
            })),
            vertex: VertexState {
                module: &wgpu_base.shader("fullscreen.vert"),
                entry_point: "main",
                buffers: &[],
            },
            fragment: Some(FragmentState {
                module: &wgpu_base.shader("shader.frag"),
                entry_point: "main",
                targets: &[ColorTargetState {
                    format: desc.format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrite::ALL,
                }],
            }),
            primitive: PrimitiveState {
                cull_mode: Some(Face::Back),
                ..Default::default()
            },
            depth_stencil: None,
            multisample: Default::default(),
        });

        Self {
            render_pipeline,
            color_index: 0,
            // texture,
            // texture_view,
            // texture_sampler,
            texture_bind_group: bind,
        }
    }
}

impl WgpuWindowedRender for App {
    fn render<'a>(&'a mut self, _: &WgpuWindowed<'_>, render_pass: &mut RenderPass<'a>) {
        let color: [f32; 3] = COLORS[self.color_index];
        self.color_index = (self.color_index + 1) % COLORS.len();
        // bytemuck might break on big endian machines

        render_pass.set_pipeline(&self.render_pipeline);
        // render_pass.set_push_constants(ShaderStage::all(), 0, bytes_of(&color));
        render_pass.set_bind_group(0, &self.texture_bind_group, &[]);
        render_pass.draw(0..3, 0..1);
    }

    fn render2(&mut self, wgpu_windowed: &WgpuWindowed<'_>, encoder: &mut wgpu::CommandEncoder) {}
}

impl ImguiWgpuRender for App {
    fn render_ui(&mut self, ui: &mut ::imgui::Ui<'_>) {
        ui.show_demo_window(&mut false);
    }
}

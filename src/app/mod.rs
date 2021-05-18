use bytemuck::bytes_of;
use wgpu::{
    BlendState, ColorTargetState, ColorWrite, Face, FragmentState, PipelineLayoutDescriptor,
    PrimitiveState, PushConstantRange, RenderPass, RenderPipeline, RenderPipelineDescriptor,
    ShaderStage, TextureFormat, VertexState,
};

use crate::imgui::ImguiWgpuRender;
use crate::util::CreateFromWgpu;
use crate::wgpu::{WgpuBase, WgpuWindowed, WgpuWindowedRender};

const COLORS: [[f32; 3]; 4] = [
    [0.0, 0.0, 0.0],
    [1.0, 0.0, 0.0],
    [0.0, 1.0, 0.0],
    [0.0, 0.0, 1.0],
];

pub struct App {
    render_pipeline: RenderPipeline,
    color_index: usize,
}

impl CreateFromWgpu for App {
    fn new(wgpu_base: &WgpuBase) -> Self {
        let device = &wgpu_base.device;

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&device.create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Default::default(),
                bind_group_layouts: Default::default(),
                push_constant_ranges: &[PushConstantRange {
                    stages: ShaderStage::all(),
                    range: 0..(4 * 3),
                }],
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
                    format: TextureFormat::Bgra8Unorm, // todo get from wgpu_windowed.swapchain_desc? // update: it crashes if it isnt lol
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
        }
    }
}

impl WgpuWindowedRender for App {
    fn render<'a>(&'a mut self, _: &WgpuWindowed<'_>, render_pass: &mut RenderPass<'a>) {
        let color: [f32; 3] = COLORS[self.color_index];
        self.color_index = (self.color_index + 1) % COLORS.len();
        // bytemuck might break on big endian machines

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_push_constants(ShaderStage::all(), 0, bytes_of(&color));
        render_pass.draw(0..3, 0..1);
    }
}

impl ImguiWgpuRender for App {
    fn render_ui(&mut self, ui: &mut ::imgui::Ui<'_>) {
        ui.show_demo_window(&mut false);
    }
}

use bytemuck::{Pod, Zeroable};
use wgpu::{
    BindGroup, BlendState, ColorTargetState, ColorWrite, CommandEncoder, Face, FragmentState,
    PipelineLayoutDescriptor, PrimitiveState, PushConstantRange, RenderPass, RenderPipeline,
    RenderPipelineDescriptor, ShaderStage, TextureFormat, TextureUsage, VertexState,
};

use crate::imgui::ImguiWgpuRender;
use crate::util::{texture_size, CreateFromWgpu, TextureDesc, TextureResult};
use crate::wgpu::{WgpuBase, WgpuWindowed, WgpuWindowedRender};

const COLORS: [[f32; 3]; 4] = [
    [0.0, 0.0, 0.0],
    [1.0, 0.0, 0.0],
    [0.0, 1.0, 0.0],
    [0.0, 0.0, 1.0],
];

#[derive(Default)]
struct PushConstants {
    width: u32,
    pixels: u32,
    front: bool,
}

impl PushConstants {
    fn as_bytes(&self) -> [u8; 12] {
        #[derive(Copy, Clone, Pod, Zeroable)]
        #[repr(C)]
        struct PushConstantsAligned {
            width: u32,
            pixels: u32,
            front: u32,
        }

        let aligned = PushConstantsAligned {
            width: self.width,
            pixels: self.pixels,
            front: self.front as _,
        };

        bytemuck::cast(aligned)
    }
}

pub struct App {
    render_pipeline: RenderPipeline,
    color_index: usize,
    texture_bind_group: BindGroup,
    push_constants: PushConstants,
}

impl CreateFromWgpu for App {
    fn new(wgpu_base: &WgpuBase, swapchain_desc: &TextureDesc) -> Self {
        let desc = TextureDesc {
            width: 200,
            height: 200,
            format: TextureFormat::R8Unorm,
        };
        let desc = desc.into_2d(TextureUsage::COPY_DST | TextureUsage::SAMPLED);

        let device = &wgpu_base.device;

        let num_bytes = texture_size(&desc);
        let data: Vec<u8> = (0..num_bytes).into_iter().map(|x| (x * 85) as _).collect();

        let data2: Vec<u8> = std::iter::repeat([0, 255, 0, 255].iter())
            .flatten()
            .copied()
            .take(num_bytes as _)
            .collect();

        let TextureResult {
            bind_layout, bind, ..
        } = wgpu_base.texture(&desc, Default::default(), Some(Some(&data)));

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&device.create_pipeline_layout(&PipelineLayoutDescriptor {
                bind_group_layouts: &[&bind_layout],
                push_constant_ranges: &[PushConstantRange {
                    stages: ShaderStage::FRAGMENT,
                    range: 0..(4 * 3),
                }],
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
                    format: swapchain_desc.format,
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
            texture_bind_group: bind,
            push_constants: PushConstants {
                front: true,
                ..Default::default()
            },
        }
    }
}

impl WgpuWindowedRender for App {
    fn render<'a>(&'a mut self, _: &WgpuWindowed<'_>, render_pass: &mut RenderPass<'a>) {
        let color: [f32; 3] = COLORS[self.color_index];
        self.color_index = (self.color_index + 1) % COLORS.len();
        // bytemuck might break on big endian machines

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_push_constants(ShaderStage::FRAGMENT, 0, &self.push_constants.as_bytes());
        render_pass.set_bind_group(0, &self.texture_bind_group, &[]);
        render_pass.draw(0..3, 0..1);
    }

    fn render_encoder(
        &mut self,
        wgpu_windowed: &WgpuWindowed<'_>,
        encoder: &mut CommandEncoder,
        after: bool,
    ) {
        self.push_constants.width = wgpu_windowed.desc().width;
    }
}

impl ImguiWgpuRender for App {
    fn render_ui(&mut self, ui: &mut imgui::Ui<'_>) {
        use imgui::{im_str, Drag, Window};

        // ui.show_demo_window(&mut false);

        let PushConstants {
            width,
            pixels,
            front,
        } = &mut self.push_constants;

        Window::new(im_str!("App"))
            .always_auto_resize(true)
            .build(ui, || {
                ui.push_item_width(70.0);
                ui.label_text(im_str!("width"), &im_str!("{}", width));
                Drag::new(im_str!("pixels"))
                    .range(0..=*width)
                    .speed(4.0)
                    .build(ui, pixels);
                ui.checkbox(im_str!("front"), front);
            });
    }
}

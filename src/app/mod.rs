use bytemuck::{Pod, Zeroable};
use wgpu::{
    BindGroup, BufferBindingType, BufferUsage, ColorTargetState, ColorWrite, CommandEncoder,
    ComputePipeline, ComputePipelineDescriptor, Face, FragmentState, PipelineLayoutDescriptor,
    PrimitiveState, PushConstantRange, RenderPass, RenderPipeline, RenderPipelineDescriptor,
    ShaderStage, StorageTextureAccess, TextureFormat, TextureUsage, VertexState,
};

use crate::imgui::ImguiWgpuRender;
use crate::util::{CreateFromWgpu, InitType, TextureDesc};
use crate::wgpu::{
    BindGroupEntry, BindGroupResult, BufferDesc, TextureResult, WgpuBase, WgpuWindowed,
    WgpuWindowedRender,
};

const COLORS: [[f32; 3]; 4] = [
    [0.0, 0.0, 0.0],
    [1.0, 0.0, 0.0],
    [0.0, 1.0, 0.0],
    [0.0, 0.0, 1.0],
];

struct PushConstants {
    desc: TextureDesc,
    pixels: u32,
    front: bool,
}

impl PushConstants {
    fn as_bytes(&self) -> [u8; 16] {
        #[derive(Copy, Clone, Pod, Zeroable)]
        #[repr(C)]
        struct PushConstantsAligned {
            width: u32,
            height: u32,
            pixels: u32,
            front: u32,
        }

        let aligned = PushConstantsAligned {
            width: self.desc.width,
            height: self.desc.height,
            pixels: self.pixels,
            front: self.front as _,
        };

        bytemuck::cast(aligned)
    }
}

pub struct App {
    render_pipeline: RenderPipeline,
    compute_pipeline: ComputePipeline,
    color_index: usize,
    texture_bind_group: BindGroup,
    comp_tex_bind_group: BindGroup,
    push_constants: PushConstants,
}

impl CreateFromWgpu for App {
    fn new(wgpu_base: &WgpuBase, swapchain_desc: &TextureDesc) -> Self {
        let desc = TextureDesc {
            format: TextureFormat::Rgba8Uint,
            ..swapchain_desc.clone()
        };
        let desc =
            desc.into_2d(TextureUsage::COPY_DST | TextureUsage::SAMPLED | TextureUsage::STORAGE);

        let device = &wgpu_base.device;

        let TextureResult { view: tex_view, .. } =
            wgpu_base.texture(&desc, InitType::Repeated(&[0, 255, 0, 255]));

        let BindGroupResult {
            layout: tex_bind_layout,
            bind: texture_bind_group,
        } = wgpu_base.bind_group(&[BindGroupEntry::Texture {
            storage: None,
            desc: desc.clone(),
            view: &tex_view,
        }]);

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&device.create_pipeline_layout(&PipelineLayoutDescriptor {
                bind_group_layouts: &[&tex_bind_layout],
                push_constant_ranges: &[PushConstantRange {
                    stages: ShaderStage::FRAGMENT,
                    range: 0..(4 * 4),
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
                    blend: None,
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

        let BindGroupResult {
            layout: comp_tex_bind_layout,
            bind: comp_tex_bind_group,
        } = wgpu_base.bind_group(&[BindGroupEntry::Texture {
            storage: Some(StorageTextureAccess::WriteOnly),
            desc: desc.clone(),
            view: &tex_view,
        }]);

        let compute_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: None,
            layout: Some(&device.create_pipeline_layout(&PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&comp_tex_bind_layout],
                push_constant_ranges: &[PushConstantRange {
                    stages: ShaderStage::COMPUTE,
                    range: 0..(4 * 2),
                }],
            })),
            module: &wgpu_base.shader("pixels.comp"),
            entry_point: "main",
        });

        Self {
            render_pipeline,
            compute_pipeline,
            color_index: 0,
            texture_bind_group,
            // frag_tex_bind_group,
            comp_tex_bind_group,
            // comp_config_buffer,
            // comp_data_buffer,
            // comp_bind_group,
            push_constants: PushConstants {
                front: true,
                desc: swapchain_desc.clone(),
                pixels: 0,
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
        if after {
            return;
        }

        let desc = wgpu_windowed.desc();
        let TextureDesc { width, height, .. } = desc;

        self.push_constants.desc = desc;

        let comp_push_consts = [width, height];
        let comp_push_consts = bytemuck::bytes_of(&comp_push_consts);

        let mut compute_pass = encoder.begin_compute_pass(&Default::default());
        compute_pass.set_pipeline(&self.compute_pipeline);
        compute_pass.set_bind_group(0, &self.comp_tex_bind_group, &[]);
        // compute_pass.set_bind_group(1, &self.comp_bind_group, &[]);
        compute_pass.set_push_constants(0, comp_push_consts);
        compute_pass.dispatch(800 / 8 + 1, 600 / 8 + 1, 1);
    }
}

impl ImguiWgpuRender for App {
    fn render_ui(&mut self, ui: &mut imgui::Ui<'_>) {
        use imgui::{im_str, Drag, Window};

        // ui.show_demo_window(&mut false);

        let PushConstants {
            desc: TextureDesc { width, .. },
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

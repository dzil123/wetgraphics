use bytemuck::{Pod, Zeroable};
use wgpu::{CommandEncoder, RenderPass, StorageTextureAccess, TextureFormat, TextureUsage};

use crate::imgui::ImguiWgpuRender;
use crate::util::{CreateFromWgpu, InitType, TextureDesc};
use crate::wgpu::{
    BindGroupEntry, ComputePipelineDesc, FullComputePipeline, FullRenderPipeline, PipelineExt,
    RenderPipelineDesc, TextureResult, WgpuBase, WgpuWindowed, WgpuWindowedRender,
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
    render_pipeline: FullRenderPipeline,
    compute_pipeline: FullComputePipeline,
    color_index: usize,
    push_constants: PushConstants,
}

impl CreateFromWgpu for App {
    fn new(wgpu_base: &mut WgpuBase, swapchain_desc: &TextureDesc) -> Self {
        let desc = TextureDesc {
            format: TextureFormat::Rgba8Uint,
            ..swapchain_desc.clone()
        };
        let desc =
            desc.into_2d(TextureUsage::COPY_DST | TextureUsage::SAMPLED | TextureUsage::STORAGE);

        let TextureResult { view: tex_view, .. } =
            wgpu_base.texture(&desc, InitType::Repeated(&[0, 255, 0, 255]));

        let render_pipeline = wgpu_base.render_pipeline(RenderPipelineDesc {
            bind_groups: vec![wgpu_base.bind_group(&[BindGroupEntry::Texture {
                storage: None,
                desc: desc.clone(),
                view: &tex_view,
            }])],
            shader: "shader.frag",
            target: swapchain_desc.format.into(),
            push_constants: Some(4 * 4),
        });

        let compute_pipeline = wgpu_base.compute_pipeline(ComputePipelineDesc {
            bind_groups: vec![wgpu_base.bind_group(&[BindGroupEntry::Texture {
                storage: Some(StorageTextureAccess::WriteOnly),
                desc: desc.clone(),
                view: &tex_view,
            }])],
            shader: "pixels.comp",
            push_constants: Some(4 * 2),
        });

        Self {
            render_pipeline,
            compute_pipeline,
            color_index: 0,
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

        render_pass.begin(&self.render_pipeline);
        render_pass.pushc(&self.push_constants.as_bytes());
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
        compute_pass.begin(&self.compute_pipeline);
        compute_pass.pushc(comp_push_consts);
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

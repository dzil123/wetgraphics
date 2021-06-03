use bytemuck::{Pod, Zeroable};
use imgui::{ColorEdit, ColorEditFlags, SliderFlags};
use wgpu::{CommandEncoder, RenderPass, StorageTextureAccess, TextureFormat, TextureUsage};

use crate::util::{align_to, CreateFromWgpu, InitType, SamplerDesc, TextureDesc};
use crate::wgpu::{
    BindGroupEntry, ComputePipelineDesc, FullComputePipeline, FullRenderPipeline, PipelineExt,
    RenderPipelineDesc, TextureResult, WgpuBase, WgpuWindowed, WgpuWindowedRender,
};
use crate::{imgui::ImguiWgpuRender, util::as_bool};

use crevice::std430::{AsStd430, Std430, UVec2, Vec3};

#[derive(AsStd430, Debug)]
struct FragmentConfig {
    foreground_color: Vec3,
    background_color: Vec3,
    flip: u32, // bool
    offset: f32,
}

#[derive(AsStd430, Debug)]
struct ComputeConfig {
    size: UVec2,
}

pub struct App {
    render_pipeline: FullRenderPipeline,
    compute_pipeline: FullComputePipeline,
    color_index: usize,
    compute_config: ComputeConfig,
    fragment_config: FragmentConfig,
}

impl CreateFromWgpu for App {
    fn new(wgpu_base: &mut WgpuBase, swapchain_desc: &TextureDesc) -> Self {
        let desc1 = TextureDesc {
            format: TextureFormat::R32Float,
            width: 400,
            height: 300,
        };
        let desc =
            desc1.into_2d(TextureUsage::COPY_DST | TextureUsage::SAMPLED | TextureUsage::STORAGE);

        let TextureResult { view: tex_view, .. } = wgpu_base.texture(
            &desc,
            InitType::Repeated(bytemuck::cast_slice(&[0.0f32, 0.5, 1.0])),
        );

        let render_pipeline = wgpu_base.render_pipeline(RenderPipelineDesc {
            bind_groups: vec![wgpu_base.bind_group(&[
                BindGroupEntry::Texture {
                    storage: None,
                    desc: desc.clone(),
                    view: &tex_view,
                },
                BindGroupEntry::Sampler {
                    desc: SamplerDesc {
                        filter: false,
                        ..Default::default()
                    },
                },
            ])],
            shader: "shader.frag",
            target: swapchain_desc.format.into(),
            push_constants: Some(FragmentConfig::std430_size_static() as _),
        });

        let compute_pipeline = wgpu_base.compute_pipeline(ComputePipelineDesc {
            bind_groups: vec![wgpu_base.bind_group(&[BindGroupEntry::Texture {
                storage: Some(StorageTextureAccess::WriteOnly),
                desc: desc.clone(),
                view: &tex_view,
            }])],
            shader: "pixels.comp",
            push_constants: Some(ComputeConfig::std430_size_static() as _),
        });

        Self {
            render_pipeline,
            compute_pipeline,
            color_index: 0,
            compute_config: ComputeConfig { size: desc1.size() },
            fragment_config: FragmentConfig {
                foreground_color: Vec3 {
                    x: 0.5,
                    y: 1.0,
                    z: 0.0,
                },
                background_color: Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 1.0,
                },
                flip: false as _,
                offset: 0.0,
            },
        }
    }
}

impl WgpuWindowedRender for App {
    fn render<'a>(&'a mut self, _: &WgpuWindowed<'_>, render_pass: &mut RenderPass<'a>) {
        render_pass.begin(&self.render_pipeline);
        render_pass.pushc(self.fragment_config.as_std430().as_bytes());
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

        let group_size = 16;
        let group_size_x = align_to(self.compute_config.size.x, group_size) / group_size;
        let group_size_y = align_to(self.compute_config.size.y, group_size) / group_size;

        let mut compute_pass = encoder.begin_compute_pass(&Default::default());
        compute_pass.begin(&self.compute_pipeline);
        compute_pass.pushc(self.compute_config.as_std430().as_bytes());
        compute_pass.dispatch(group_size_x, group_size_y, 1);
    }
}

impl ImguiWgpuRender for App {
    fn render_ui(&mut self, ui: &mut imgui::Ui<'_>) {
        use imgui::{im_str, Drag, Window};

        // ui.show_demo_window(&mut false);

        let FragmentConfig {
            foreground_color,
            background_color,
            flip,
            offset,
        } = &mut self.fragment_config;

        let foreground_color: &mut [f32; 3] = bytemuck::cast_mut(foreground_color);
        let background_color: &mut [f32; 3] = bytemuck::cast_mut(background_color);
        let flip = as_bool(flip);

        Window::new(im_str!("Fragment"))
            .always_auto_resize(true)
            .build(ui, || {
                ColorEdit::new(im_str!("Foreground"), foreground_color).build(ui);
                ColorEdit::new(im_str!("Background"), background_color).build(ui);
                ui.checkbox(im_str!("Flip"), flip);
                Drag::new(im_str!("offset"))
                    .range(0.0..=1.0)
                    .speed(0.001)
                    .display_format(im_str!("%.3f"))
                    .build(ui, offset);
            });
    }
}

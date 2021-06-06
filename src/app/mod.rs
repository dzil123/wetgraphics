use ::wgpu::{
    Buffer, BufferBindingType, BufferUsage, CommandEncoder, RenderPass, StorageTextureAccess,
    TextureFormat, TextureUsage,
};
use bytemuck::Zeroable;
use crevice::{
    std140::{AsStd140, Std140},
    std430::{AsStd430, Std430, UVec2, Vec2, Vec3},
};
use imgui::{ColorEdit, ColorEditFlags, SliderFlags};

use crate::imgui::ImguiWgpuRender;
use crate::util::{
    align_to, as_bool, group_size, CreateFromWgpu, InitType, SamplerDesc, TextureDesc,
};
use crate::wgpu::{
    BindGroupEntry, BufferDesc, ComputePipelineDesc, FullComputePipeline, FullRenderPipeline,
    PipelineExt, RenderPipelineDesc, TextureResult, WgpuBase, WgpuWindowed, WgpuWindowedRender,
};

#[derive(AsStd430, Debug)]
struct FragmentConfig {
    foreground_color: Vec3,
    background_color: Vec3,
    flip: u32, // bool
    offset: f32,
}

#[derive(AsStd140, Debug, Default)]
struct ComputeConfig {
    speed: f32,
    sensor_dist: f32,
    sensor_size: u32,
    sensor_angle: f32,
    turn_speed: f32,
}

#[derive(AsStd430, Debug)]
struct Agent {
    pos: Vec2,
    angle: f32,
}

#[derive(AsStd430, Debug)]
struct DiffuseConfig {
    attenuate: f32,
    diffuse: f32,
}

struct AgentBuffer {
    size: u32,
    agents: Vec<<Agent as AsStd430>::Std430Type>,
}

impl AgentBuffer {
    fn new(size: u32) -> Self {
        Self {
            size,
            agents: vec![Zeroable::zeroed(); size as _],
        }
    }

    fn write(&self) -> Vec<u8> {
        let mut data = Vec::new();
        let mut writer = crevice::std430::Writer::new(&mut data);

        let agents: &[<Agent as AsStd430>::Std430Type] = &self.agents;

        writer.write_std430(&self.size).unwrap();
        writer.write(agents).unwrap();

        data
    }
}

const DESC: TextureDesc = TextureDesc {
    format: TextureFormat::R32Float,
    width: 1920 / 2,
    height: 1015 / 2,
};

pub struct App {
    render_pipeline: FullRenderPipeline,
    init_compute_pipeline: FullComputePipeline,
    draw_compute_pipeline: FullComputePipeline,
    diffuse_compute_pipeline: FullComputePipeline,
    compute_config: ComputeConfig,
    compute_config_buffer: Buffer,
    fragment_config: FragmentConfig,
    diffuse_config: DiffuseConfig,
    first_run: bool,
    num_agents: u32,
}

impl CreateFromWgpu for App {
    fn new(wgpu_base: &mut WgpuBase, swapchain_desc: &TextureDesc) -> Self {
        let desc =
            DESC.into_2d(TextureUsage::COPY_DST | TextureUsage::SAMPLED | TextureUsage::STORAGE);

        let TextureResult { view: tex_view, .. } = wgpu_base.texture(
            &desc,
            // InitType::Repeated(bytemuck::cast_slice(&[0.3f32, 0.4, 0.5, 0.6, 0.7])),
            InitType::Zeros,
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

        let num_agents = 1000;
        let agent_buffer_data = AgentBuffer::new(num_agents).write();
        let agent_buffer = wgpu_base.buffer(
            BufferDesc {
                size: dbg!(agent_buffer_data.len()),
                usage: BufferUsage::STORAGE,
            },
            InitType::Data(&agent_buffer_data),
        );

        let agent_bind_buffer = BindGroupEntry::Buffer {
            ty: BufferBindingType::Storage { read_only: false },
            buffer: &agent_buffer,
        };

        let rw_tex_bind = BindGroupEntry::Texture {
            storage: Some(StorageTextureAccess::ReadWrite),
            desc,
            view: &tex_view,
        };

        let compute_config_buffer = wgpu_base.buffer(
            BufferDesc {
                size: ComputeConfig::std140_size_static(),
                usage: BufferUsage::UNIFORM | BufferUsage::COPY_DST,
            },
            InitType::Uninit,
        );

        let init_compute_pipeline = wgpu_base.compute_pipeline(ComputePipelineDesc {
            bind_groups: vec![wgpu_base.bind_group(&[agent_bind_buffer.clone()])],
            shader: "init_agents.comp",
            push_constants: Some(UVec2::std430_size_static() as _),
        });

        let draw_compute_pipeline = wgpu_base.compute_pipeline(ComputePipelineDesc {
            bind_groups: vec![wgpu_base.bind_group(&[
                rw_tex_bind.clone(),
                agent_bind_buffer.clone(),
                BindGroupEntry::Buffer {
                    ty: BufferBindingType::Uniform,
                    buffer: &compute_config_buffer,
                },
            ])],
            shader: "draw_agents.comp",
            push_constants: None,
        });

        let diffuse_compute_pipeline = wgpu_base.compute_pipeline(ComputePipelineDesc {
            bind_groups: vec![wgpu_base.bind_group(&[rw_tex_bind.clone()])],
            shader: "diffuse_pass.comp",
            push_constants: Some(DiffuseConfig::std430_size_static() as _),
        });

        Self {
            render_pipeline,
            init_compute_pipeline,
            draw_compute_pipeline,
            diffuse_compute_pipeline,
            compute_config: ComputeConfig {
                speed: 60.0,
                sensor_dist: 1.0,
                sensor_size: 2,
                sensor_angle: 30.0,
                turn_speed: 0.0,
            },
            compute_config_buffer,
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
            diffuse_config: DiffuseConfig {
                attenuate: 0.5,
                diffuse: 0.5,
            },
            first_run: true,
            num_agents,
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

        wgpu_windowed.base.queue.write_buffer(
            &self.compute_config_buffer,
            0,
            self.compute_config.as_std140().as_bytes(),
        );

        let mut compute_pass = encoder.begin_compute_pass(&Default::default());

        if self.first_run {
            self.first_run = false;

            compute_pass.begin(&self.init_compute_pipeline);
            compute_pass.pushc(DESC.size().as_bytes());
            compute_pass.dispatch(group_size(self.num_agents, 64), 1, 1); // todo: add group size to fullcomputepipeline?
        }

        let groups = DESC.group_size(16);
        compute_pass.begin(&self.diffuse_compute_pipeline);
        compute_pass.pushc(self.diffuse_config.as_std430().as_bytes());
        compute_pass.dispatch(groups.x, groups.y, 1);

        compute_pass.begin(&self.draw_compute_pipeline);
        compute_pass.dispatch(group_size(self.num_agents, 64), 1, 1);
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
                Drag::new(im_str!("Offset"))
                    .range(0.0..=1.0)
                    .speed(0.001)
                    .display_format(im_str!("%.3f"))
                    .build(ui, offset);
            });

        let ComputeConfig {
            speed,
            sensor_dist,
            sensor_size,
            sensor_angle,
            turn_speed,
        } = &mut self.compute_config;
        let DiffuseConfig { attenuate, diffuse } = &mut self.diffuse_config;

        Window::new(im_str!("Compute"))
            .always_auto_resize(true)
            .build(ui, || {
                Drag::new(im_str!("Speed")).range(0.0..).build(ui, speed);
                Drag::new(im_str!("Sensor Distance"))
                    .range(0.0..)
                    .speed(0.1)
                    .build(ui, sensor_dist);
                Drag::new(im_str!("Sensor Size"))
                    .range(0..)
                    .speed(0.1)
                    .build(ui, sensor_size);
                Drag::new(im_str!("Sensor Angle"))
                    .range(0.0..=90.0)
                    .speed(1.0)
                    .build(ui, sensor_angle);
                Drag::new(im_str!("Turn Speed"))
                    .range(0.0..=1.0)
                    .speed(0.005)
                    .flags(SliderFlags::LOGARITHMIC)
                    .build(ui, turn_speed);
                ui.new_line();
                Drag::new(im_str!("Attenuate"))
                    .range(0.0..=1.0)
                    .speed(0.005)
                    .flags(SliderFlags::LOGARITHMIC)
                    .build(ui, attenuate);
                Drag::new(im_str!("Diffuse"))
                    .range(0.0..=1.0)
                    .speed(0.005)
                    .flags(SliderFlags::LOGARITHMIC)
                    .build(ui, diffuse);
            });
    }
}

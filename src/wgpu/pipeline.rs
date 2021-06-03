use wgpu::{
    BindGroup, BindGroupLayout, ColorTargetState, ComputePass, ComputePipeline,
    ComputePipelineDescriptor, Face, FragmentState, PipelineLayout, PipelineLayoutDescriptor,
    PrimitiveState, PushConstantRange, RenderPass, RenderPipeline, RenderPipelineDescriptor,
    ShaderStage, VertexState,
};

use super::{BindGroupResult, WgpuBase};

impl WgpuBase {
    pub fn render_pipeline(&mut self, desc: RenderPipelineDesc) -> FullRenderPipeline {
        let (layout, binds) =
            self.pipeline(desc.bind_groups, desc.push_constants, ShaderStage::FRAGMENT);

        let vertex_shader = "fullscreen.vert";
        let fragment_shader = desc.shader;

        self.shader_preload(vertex_shader);
        self.shader_preload(fragment_shader);

        let device = &self.device;
        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            layout: Some(&layout),
            vertex: VertexState {
                module: self.shader(vertex_shader),
                entry_point: "main",
                buffers: &[],
            },
            fragment: Some(FragmentState {
                module: self.shader(fragment_shader),
                entry_point: "main",
                targets: &[desc.target],
            }),
            primitive: PrimitiveState {
                cull_mode: Some(Face::Back),
                ..Default::default()
            },
            depth_stencil: None,
            multisample: Default::default(),
            label: None,
        });

        FullRenderPipeline {
            pipeline,
            bind_groups: binds,
        }
    }

    pub fn compute_pipeline(&mut self, desc: ComputePipelineDesc) -> FullComputePipeline {
        let (layout, binds) =
            self.pipeline(desc.bind_groups, desc.push_constants, ShaderStage::COMPUTE);

        self.shader_preload(desc.shader);

        let device = &self.device;
        let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            layout: Some(&layout),
            module: &self.shader(desc.shader),
            entry_point: "main",
            label: None,
        });

        FullComputePipeline {
            pipeline,
            bind_groups: binds,
        }
    }

    fn pipeline(
        &self,
        bind_groups: Vec<BindGroupResult>,
        push_constants: Option<u32>,
        stages: ShaderStage,
    ) -> (PipelineLayout, Vec<BindGroup>) {
        let device = &self.device;

        let (layouts, binds): (Vec<BindGroupLayout>, Vec<BindGroup>) = bind_groups
            .into_iter()
            .map(|res| (res.layout, res.bind))
            .unzip();

        let layouts: Vec<&BindGroupLayout> = layouts.iter().collect();

        let mut pushc: &[PushConstantRange] = &[PushConstantRange {
            stages,
            range: 0..push_constants.unwrap_or_default(),
        }];

        if push_constants.is_none() {
            pushc = &[];
        }

        let layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            bind_group_layouts: &layouts,
            push_constant_ranges: pushc,
            ..Default::default()
        });

        (layout, binds)
    }
}

pub struct RenderPipelineDesc {
    pub bind_groups: Vec<BindGroupResult>,
    pub shader: &'static str,
    pub target: ColorTargetState,
    pub push_constants: Option<u32>,
}

pub struct FullRenderPipeline {
    pipeline: RenderPipeline,
    bind_groups: Vec<BindGroup>,
}

pub struct ComputePipelineDesc {
    pub bind_groups: Vec<BindGroupResult>,
    pub shader: &'static str,
    pub push_constants: Option<u32>,
}

pub struct FullComputePipeline {
    pipeline: ComputePipeline,
    bind_groups: Vec<BindGroup>,
}

pub trait PipelineExt<'a> {
    type FullPipeline;

    fn begin(&mut self, pipeline: &'a Self::FullPipeline);
    fn pushc(&mut self, data: &[u8]);
}

impl<'a> PipelineExt<'a> for RenderPass<'a> {
    type FullPipeline = FullRenderPipeline;

    fn begin(&mut self, pipeline: &'a Self::FullPipeline) {
        self.set_pipeline(&pipeline.pipeline);
        for (index, bind_group) in pipeline.bind_groups.iter().enumerate() {
            self.set_bind_group(index as _, bind_group, &[]);
        }
    }

    fn pushc(&mut self, data: &[u8]) {
        self.set_push_constants(ShaderStage::FRAGMENT, 0, data);
    }
}

impl<'a> PipelineExt<'a> for ComputePass<'a> {
    type FullPipeline = FullComputePipeline;

    fn begin(&mut self, pipeline: &'a Self::FullPipeline) {
        self.set_pipeline(&pipeline.pipeline);
        for (index, bind_group) in pipeline.bind_groups.iter().enumerate() {
            self.set_bind_group(index as _, bind_group, &[]);
        }
    }

    fn pushc(&mut self, data: &[u8]) {
        self.set_push_constants(0, data);
    }
}

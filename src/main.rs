#![allow(unused_variables, unreachable_code, dead_code, unused_imports)]
#![warn(rust_2018_idioms)]

use ::wgpu::{
    BlendState, ColorTargetState, ColorWrite, CullMode, FragmentState, PipelineLayoutDescriptor,
    PrimitiveState, RenderPass, RenderPipeline, RenderPipelineDescriptor, ShaderModule,
    TextureFormat, VertexState,
};
use winit::{
    event::{Event, WindowEvent},
    window::Window as WinitWindow,
};

use crate::imgui::ImguiWgpu;
use crate::mainloop::Mainloop;
use crate::util::WindowSize;
use crate::wgpu::{WgpuBase, WgpuWindowed};
use crate::window::Window;

mod imgui;
mod mainloop;
mod shaders;
mod util;
mod wgpu;
mod window;

struct State {
    render_pipeline: RenderPipeline,
}

impl State {
    pub fn new(wgpu_base: &WgpuBase) -> Self {
        let device = &wgpu_base.device;

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&device.create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Default::default(),
                bind_group_layouts: Default::default(),
                push_constant_ranges: Default::default(),
            })),
            vertex: VertexState {
                module: &wgpu_base.shader("shader.vert"),
                entry_point: "main",
                buffers: &[],
            },
            fragment: Some(FragmentState {
                module: &wgpu_base.shader("shader.frag"),
                entry_point: "main",
                targets: &[ColorTargetState {
                    format: TextureFormat::Bgra8Unorm, // todo get from wgpu_windowed.swapchain_desc?
                    alpha_blend: BlendState::REPLACE,
                    color_blend: BlendState::REPLACE,
                    write_mask: ColorWrite::ALL,
                }],
            }),
            primitive: PrimitiveState {
                cull_mode: CullMode::Back,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: Default::default(),
        });

        Self { render_pipeline }
    }

    pub fn render<'a, 'b>(&'a mut self, _wgpu_base: &WgpuBase, render_pass: &mut RenderPass<'b>)
    where
        'a: 'b,
    {
        // render_pass.set_pipeline(&self.render_pipeline);
        // render_pass.draw(0..3, 0..1);
    }

    // pub fn render(&mut self, _wgpu_base: &WgpuBase, render_pass: &mut RenderPass) {
    //     render_pass.set_pipeline(&self.render_pipeline);
    //     render_pass.draw(0..3, 0..1);
    // }
}

struct WgpuWindowMainloop<'a> {
    wgpu_window: WgpuWindowed<'a>,
    // imgui: ImguiWgpu<'a>,
    state: State,
}

impl<'a> WgpuWindowMainloop<'a> {
    pub fn new(window: &'a WinitWindow) -> Self {
        let wgpu_window = WgpuWindowed::new(window);
        // let imgui = ImguiWgpu::new(window, &wgpu_window);
        let state = State::new(&wgpu_window.base);
        Self {
            wgpu_window,
            // imgui,
            state,
        }
    }
}

impl<'a> Mainloop for WgpuWindowMainloop<'a> {
    fn event(&mut self, _event: &Event<'_, ()>) {}

    fn input(&mut self, _event: &WindowEvent<'_>) {}

    fn render<'x>(&'x mut self) {
        // needed to convince compiler that we aren't &mut self twice
        // let Self {
        //     wgpu_window,
        //     // imgui,
        //     state: &'a mut State,
        // } = self;

        self.wgpu_window
            .render(move |_, wgpu_base, render_pass: RenderPass<'_>| {
                // let mut render_pass = None.unwrap();
                self.state.render(wgpu_base, &mut render_pass);
                // drop(state);
                drop(render_pass);
                // imgui.render(&mut render_pass, |ui| ui.show_demo_window(&mut false));
            });
    }

    fn resize(&mut self, size: WindowSize) {
        self.wgpu_window.resize(Some(size));
    }

    fn ignore_keyboard(&self) -> bool {
        false
    }
}

fn main() {
    wgpu_subscriber::initialize_default_subscriber(None);
    let (window, winit_window) = Window::new();
    let mainloop = WgpuWindowMainloop::new(&winit_window);
    window.run(&winit_window, mainloop);
}

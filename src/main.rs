#![allow(unused_variables, unreachable_code, dead_code, unused_imports)]
#![deny(rust_2018_idioms)]

use ::wgpu::{
    BlendState, ColorTargetState, ColorWrite, CullMode, FragmentState, PipelineLayoutDescriptor,
    PrimitiveState, RenderPass, RenderPipeline, RenderPipelineDescriptor, TextureFormat,
    VertexState,
};
use winit::{
    event::{Event, WindowEvent},
    window::Window as WinitWindow,
};

use crate::imgui::{ImguiWgpu, ImguiWgpuRender};
use crate::mainloop::Mainloop;
use crate::util::WindowSize;
use crate::wgpu::{WgpuBase, WgpuWindowed, WgpuWindowedRender};
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

trait CreateFromWgpu {
    fn new(wgpu_base: &WgpuBase) -> Self;
}

impl CreateFromWgpu for State {
    fn new(wgpu_base: &WgpuBase) -> Self {
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
}

impl WgpuWindowedRender for State {
    fn render<'a>(&'a mut self, _: &WgpuWindowed<'_>, render_pass: &mut RenderPass<'a>) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.draw(0..3, 0..1);
    }
}

impl ImguiWgpuRender for State {
    fn render_ui(&mut self, ui: &mut ::imgui::Ui<'_>) {
        ui.show_demo_window(&mut false);
    }
}

struct WgpuWindowMainloop<'a, T> {
    wgpu_window: WgpuWindowed<'a>,
    state: T,
}

impl<'a, T> WgpuWindowMainloop<'a, T>
where
    T: CreateFromWgpu,
{
    pub fn new(window: &'a WinitWindow) -> Self {
        let wgpu_window = WgpuWindowed::new(window);
        let state = T::new(&wgpu_window.base);
        Self { wgpu_window, state }
    }
}

impl<'a, T> Mainloop for WgpuWindowMainloop<'a, T>
where
    T: WgpuWindowedRender,
{
    fn event(&mut self, _event: &Event<'_, ()>) {}

    fn input(&mut self, _event: &WindowEvent<'_>) {}

    fn render<'x>(&'x mut self) {
        self.wgpu_window.render(&mut self.state);
    }

    fn resize(&mut self, size: WindowSize) {
        self.wgpu_window.resize(Some(size));
    }

    fn ignore_keyboard(&self) -> bool {
        false
    }
}

struct WgpuImguiWindowMainloop<'a, T> {
    wgpu_window: WgpuWindowed<'a>,
    imgui: ImguiWgpu<'a>,
    state: T,
}

impl<'a, T> WgpuImguiWindowMainloop<'a, T>
where
    T: CreateFromWgpu,
{
    pub fn new(window: &'a WinitWindow) -> Self {
        let wgpu_window = WgpuWindowed::new(window);
        let imgui = ImguiWgpu::new(window, &wgpu_window);
        let state = T::new(&wgpu_window.base);
        Self {
            wgpu_window,
            imgui,
            state,
        }
    }
}

impl<'a, T> Mainloop for WgpuImguiWindowMainloop<'a, T>
where
    T: WgpuWindowedRender + ImguiWgpuRender,
{
    fn event(&mut self, event: &Event<'_, ()>) {
        self.imgui.base.event(event);
    }

    fn input(&mut self, _event: &WindowEvent<'_>) {}

    fn render<'x>(&'x mut self) {
        self.wgpu_window
            .render(&mut self.imgui.partial_render(&mut self.state));
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
    // let mainloop = WgpuWindowMainloop::<State>::new(&winit_window);
    let mainloop = WgpuImguiWindowMainloop::<State>::new(&winit_window);
    window.run(&winit_window, mainloop);
}

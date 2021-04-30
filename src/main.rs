#![allow(unused_variables, unreachable_code, dead_code, unused_imports)]
#![deny(rust_2018_idioms)]

use ::wgpu::{
    util::RenderEncoder, BlendState, ColorTargetState, ColorWrite, CullMode, FragmentState,
    PipelineLayoutDescriptor, PrimitiveState, PushConstantRange, RenderPass, RenderPipeline,
    RenderPipelineDescriptor, ShaderStage, TextureFormat, VertexState,
};
use bytemuck::bytes_of;
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
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

const COLORS: [[f32; 3]; 4] = [
    [0.0, 0.0, 0.0],
    [1.0, 0.0, 0.0],
    [0.0, 1.0, 0.0],
    [0.0, 0.0, 1.0],
];

struct State {
    render_pipeline: RenderPipeline,
    color_index: usize,
}

// todo: add parameter for textureformat, then create CreateFromWgpuWindowed that gets the format from swapchain_desc
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

        Self {
            render_pipeline,
            color_index: 0,
        }
    }
}

// O = 4 * I
fn vec<const I: usize, const O: usize>(x: [f32; I]) -> [u8; O] {
    let mut ret = [0u8; O];

    for (val, arr) in x.iter().zip(ret.chunks_exact_mut(4)) {
        arr.copy_from_slice(&val.to_le_bytes())
    }

    ret
}

fn vec4(x: [f32; 4]) -> [u8; 4 * 4] {
    let mut ret = [0u8; 4 * 4];

    x.iter()
        .zip(ret.chunks_exact_mut(4))
        .for_each(|(val, arr)| arr.copy_from_slice(&val.to_le_bytes()));

    ret
}

impl WgpuWindowedRender for State {
    fn render<'a>(&'a mut self, _: &WgpuWindowed<'_>, render_pass: &mut RenderPass<'a>) {
        let color: [f32; 3] = COLORS[self.color_index];
        self.color_index = (self.color_index + 1) % COLORS.len();
        // bytemuck might break on big endian machines

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_push_constants(ShaderStage::all(), 0, bytes_of(&color));
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

// todo: create a trait (&self) -> (width, height:u32, format: TextureFormat) and impl for wgpu::SwapChainDescriptor
// to genericize rendering to swapchain or texture

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

    fn input(&mut self, event: &WindowEvent<'_>) {
        if let WindowEvent::KeyboardInput {
            input:
                KeyboardInput {
                    virtual_keycode: Some(key),
                    state: ElementState::Pressed,
                    ..
                },
            ..
        } = event
        {
            match key {
                VirtualKeyCode::S => self.imgui.base.suspend(),
                VirtualKeyCode::E => self.imgui.base.enable(),
                _ => {}
            }
        }
    }

    fn render<'x>(&'x mut self) {
        self.wgpu_window
            .render(&mut self.imgui.partial_render(&mut self.state));
    }

    fn resize(&mut self, size: WindowSize) {
        self.wgpu_window.resize(Some(size));
    }

    fn ignore_keyboard(&self) -> bool {
        self.imgui
            .base
            .context
            .get_ref()
            .map(|imgui| imgui.io().want_capture_keyboard)
            .unwrap_or(false)
    }
}

fn main() {
    wgpu_subscriber::initialize_default_subscriber(None);
    let (window, winit_window) = Window::new();
    // let mainloop = WgpuWindowMainloop::<State>::new(&winit_window);
    let mainloop = WgpuImguiWindowMainloop::<State>::new(&winit_window);
    window.run(&winit_window, mainloop);
}

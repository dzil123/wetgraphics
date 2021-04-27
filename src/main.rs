// #![allow(unused_variables, unreachable_code, dead_code)]

use mainloop::Mainloop;
use util::WindowSize;
use wgpu::RenderPass;
use wgpu_base::WgpuBase;
use wgpu_windowed::WgpuWindowed;
use window::Window;
use winit::window::Window as WinitWindow;

mod imgui_wgpu;
mod imgui_windowed;
mod mainloop;
mod shaders;
mod util;
mod wgpu_base;
mod wgpu_windowed;
mod window;

struct State;

impl State {
    pub fn render(
        &mut self,
        _wgpu_windowed: &WgpuWindowed,
        _wgpu_base: &WgpuBase,
        _render_pass: &mut RenderPass,
    ) {
    }
}

struct WgpuWindowMainloop<'a> {
    wgpu_window: WgpuWindowed<'a>,
    state: State,
}

impl<'a> WgpuWindowMainloop<'a> {
    pub fn new(window: &'a WinitWindow) -> Self {
        let wgpu_window = WgpuWindowed::new(window);
        Self {
            wgpu_window,
            state: State,
        }
    }
}

impl<'a> Mainloop for WgpuWindowMainloop<'a> {
    fn event(&mut self, _event: &winit::event::Event<()>) {}

    fn input(&mut self, _event: &winit::event::WindowEvent) {}

    fn render(&mut self) {
        let Self { wgpu_window, state } = self; // needed to convince compiler that we aren't &mut self twice
        wgpu_window.render(|a, b, c| state.render(a, b, c));
    }

    fn resize(&mut self, size: WindowSize) {
        self.wgpu_window.resize(Some(size));
    }

    fn ignore_keyboard(&self) -> bool {
        false
    }
}

fn main() {
    let (window, winit_window) = Window::new();
    let mainloop = WgpuWindowMainloop::new(&winit_window);
    window.run(&winit_window, mainloop);
}

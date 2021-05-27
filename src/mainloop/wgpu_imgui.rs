use std::time::Duration;

use winit::{
    event::{Event, VirtualKeyCode},
    window::Window,
};

use crate::imgui::{ImguiWgpu, ImguiWgpuRender};
use crate::util::{CreateFromWgpu, WindowSize};
use crate::wgpu::{WgpuWindowed, WgpuWindowedRender};

use super::Mainloop;

pub struct WgpuImguiWindowMainloop<'a, T> {
    pub(super) wgpu_window: WgpuWindowed<'a>,
    pub(super) imgui: ImguiWgpu<'a>,
    pub(super) state: T,
}

impl<'a, T> WgpuImguiWindowMainloop<'a, T>
where
    T: CreateFromWgpu,
{
    pub fn new(window: &'a Window) -> Self {
        let wgpu_window = WgpuWindowed::new(window);
        let imgui = ImguiWgpu::new(window, &wgpu_window);
        let state = T::new(&wgpu_window.base, &wgpu_window.desc());
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

    fn keyboard(&mut self, key: VirtualKeyCode) {
        match key {
            VirtualKeyCode::F5 => self.imgui.base.suspend(),
            VirtualKeyCode::F6 => self.imgui.base.enable(),
            _ => {}
        }
    }

    fn update(&mut self, delta: Duration) {
        if let Some(context) = self.imgui.base.context.get() {
            context.io_mut().update_delta_time(delta);
        }
    }

    fn render(&mut self) {
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
            .map_or(false, |imgui| imgui.io().want_capture_keyboard)
    }
}

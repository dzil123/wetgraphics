mod wgpu_imgui;

use std::time::Duration;

use crate::util::WindowSize;
use winit::event::{Event, WindowEvent};

pub use wgpu_imgui::WgpuImguiWindowMainloop;

pub trait Mainloop {
    fn event(&mut self, _event: &Event<'_, ()>) {}
    fn input(&mut self, _event: &WindowEvent<'_>) {}
    fn update(&mut self, _delta: Duration) {}
    fn render(&mut self) {}
    fn resize(&mut self, _size: WindowSize) {}

    fn ignore_keyboard(&self) -> bool {
        false
    }
}

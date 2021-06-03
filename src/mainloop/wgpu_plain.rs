use winit::window::Window;

use crate::util::{CreateFromWgpu, WindowSize};
use crate::wgpu::{WgpuWindowed, WgpuWindowedRender};

use super::Mainloop;

pub struct WgpuWindowMainloop<'a, T> {
    wgpu_window: WgpuWindowed<'a>,
    state: T,
}

impl<'a, T> WgpuWindowMainloop<'a, T>
where
    T: CreateFromWgpu,
{
    pub fn new(window: &'a Window) -> Self {
        let mut wgpu_window = WgpuWindowed::new(window);
        let desc = wgpu_window.desc();
        let state = T::new(&mut wgpu_window.base, &desc);
        Self { wgpu_window, state }
    }
}

impl<'a, T> Mainloop for WgpuWindowMainloop<'a, T>
where
    T: WgpuWindowedRender,
{
    fn render(&mut self) {
        self.wgpu_window.render(&mut self.state);
    }

    fn resize(&mut self, size: WindowSize) {
        self.wgpu_window.resize(Some(size));
    }
}

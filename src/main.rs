#![allow(unused_variables, unreachable_code, dead_code, unused_imports)]
#![deny(rust_2018_idioms, private_in_public)]

use mainloop::WgpuImguiWindowMainloop;
use util::CreateFromWgpu;

use crate::window::Window;

mod app;
mod imgui;
mod mainloop;
mod shaders;
mod util;
mod wgpu;
mod window;

fn main() {
    wgpu_subscriber::initialize_default_subscriber(None);
    let (window, winit_window) = Window::new();
    let mainloop = WgpuImguiWindowMainloop::<app::App>::new(&winit_window);
    window.run(&winit_window, mainloop);
}

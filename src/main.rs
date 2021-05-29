#![allow(unused_variables, unreachable_code, dead_code, unused_imports)]
#![deny(rust_2018_idioms, private_in_public)]

use std::env;

use crate::mainloop::{WgpuImguiWindowMainloop, WgpuScreenshot, WgpuWindowMainloop};
use crate::util::CreateFromWgpu;
use crate::window::Window;

mod app;
mod imgui;
mod mainloop;
mod shaders;
mod util;
mod wgpu;
mod window;

// type MainloopImpl<'a, T> = WgpuWindowMainloop<'a, T>;
// type MainloopImpl<'a, T> = WgpuImguiWindowMainloop<'a, T>;
type MainloopImpl<'a, T> = WgpuScreenshot<'a, T>;

fn main() {
    env::set_var("RUST_LOG", env::var("RUST_LOG").unwrap_or("warn".into()));
    wgpu_subscriber::initialize_default_subscriber(None);
    let (window, winit_window) = Window::new();
    let mainloop = MainloopImpl::<app::App>::new(&winit_window);
    window.run(&winit_window, mainloop);
}

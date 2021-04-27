use wgpu::{Instance, Surface};

pub trait SafeWgpuSurface {
    fn create_surface(&self, instance: &Instance) -> Surface;
}

// impl SafeWgpuSurface for () {
//     fn create_surface(&self, instance: &Instance) -> Surface {
//         unreachable!()
//     }
// }

pub type WindowSize = winit::dpi::PhysicalSize<u32>;

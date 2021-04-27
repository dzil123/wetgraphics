use wgpu::{Instance, Surface};

pub trait SafeWgpuSurface {
    fn create_surface(&self, instance: &Instance) -> Surface;
}

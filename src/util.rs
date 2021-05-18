use wgpu::{Instance, Surface};

use crate::wgpu::WgpuBase;

pub trait SafeWgpuSurface {
    fn create_surface(&self, instance: &Instance) -> Surface;
}

pub type WindowSize = winit::dpi::PhysicalSize<u32>;

// todo: add parameter for textureformat, then create CreateFromWgpuWindowed that gets the format from swapchain_desc

pub trait CreateFromWgpu {
    fn new(wgpu_base: &WgpuBase) -> Self;
}

// todo: create a trait (&self) -> (width, height:u32, format: TextureFormat) and impl for wgpu::SwapChainDescriptor
// to genericize rendering to swapchain or texture

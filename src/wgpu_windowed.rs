use wgpu::RenderPass;
use winit::window::Window;

use crate::{util::WindowSize, wgpu_base::WgpuBase};

// type Input =

// should this store window?
pub struct WgpuWindowed<'a> {
    pub base: WgpuBase,
    pub surface: wgpu::Surface,
    pub swap_chain_desc: wgpu::SwapChainDescriptor,
    pub swap_chain: wgpu::SwapChain,
    pub window: &'a Window,
}

impl<'a> WgpuWindowed<'a> {
    pub fn new(window: &'a Window) -> Self {
        let (base, surface) = WgpuBase::new_surface(window);

        let size = window.inner_size();

        let swap_chain_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8Unorm, // adapter.get_swap_chain_preferred_format?
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        let swap_chain = base.device.create_swap_chain(&surface, &swap_chain_desc);

        Self {
            base,
            surface,
            swap_chain_desc,
            swap_chain,
            window,
        }
    }

    pub fn resize(&mut self, size: Option<WindowSize>) {
        if let Some(size) = size {
            self.swap_chain_desc.width = size.width;
            self.swap_chain_desc.height = size.height;
        }

        self.swap_chain = self
            .base
            .device
            .create_swap_chain(&self.surface, &self.swap_chain_desc);
    }

    pub fn render<T>(&mut self, func: T)
    where
        T: FnOnce(&Self, &WgpuBase, &mut RenderPass),
    {
        let texture = match self.swap_chain.get_current_frame() {
            Ok(frame) => frame.output,
            Err(err) => {
                return match err {
                    wgpu::SwapChainError::Lost => self.resize(None),
                    wgpu::SwapChainError::OutOfMemory => panic!("{}", err),
                    _ => {}
                }
            }
        };

        let new_func = |wgpu_base: &WgpuBase, renderpass: &mut RenderPass| {
            func(self, wgpu_base, renderpass);
        };

        self.base.render(&texture.view, new_func);
    }
}

use wgpu::{
    CommandEncoder, PresentMode, RenderPass, Surface, SwapChain, SwapChainDescriptor,
    SwapChainError, TextureFormat, TextureUsage,
};
use winit::window::Window;

use super::wgpu_base::{WgpuBase, WgpuBaseRender};
use crate::util::{TextureDesc, WindowSize};

// should this store window?
pub struct WgpuWindowed<'a> {
    pub base: WgpuBase,
    pub surface: Surface,
    pub swap_chain_desc: SwapChainDescriptor,
    pub swap_chain: SwapChain,
    pub window: &'a Window,
}

impl<'a> WgpuWindowed<'a> {
    pub fn new(window: &'a Window) -> Self {
        let (base, surface) = WgpuBase::new_surface(window);

        let size = window.inner_size();

        let swap_chain_desc = SwapChainDescriptor {
            usage: TextureUsage::RENDER_ATTACHMENT,
            format: TextureFormat::Bgra8Unorm, // adapter.get_swap_chain_preferred_format? // srgb causes linear colors passed in as push constants to be incorrectly lightened
            width: size.width,
            height: size.height,
            present_mode: PresentMode::Fifo,
            // present_mode: PresentMode::Immediate,
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

    // warning: does not update with resizes
    pub fn desc(&self) -> TextureDesc {
        (&self.swap_chain_desc).into()
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

    pub fn render<T>(&mut self, target: &mut T)
    where
        T: WgpuWindowedRender,
    {
        let texture = match self.swap_chain.get_current_frame() {
            Ok(frame) => frame.output,
            Err(err) => {
                return match err {
                    SwapChainError::Lost => self.resize(None),
                    SwapChainError::OutOfMemory => panic!("{}", err),
                    _ => {}
                }
            }
        };

        let mut helper = HelperRenderTarget {
            wgpu_windowed: &self,
            inner: target,
        };

        self.base.render(&texture.view, &mut helper);
    }
}

struct HelperRenderTarget<'a, T> {
    wgpu_windowed: &'a WgpuWindowed<'a>,
    inner: &'a mut T,
}

impl<'a, T> WgpuBaseRender for HelperRenderTarget<'a, T>
where
    T: WgpuWindowedRender,
{
    fn render<'b>(&'b mut self, _: &WgpuBase, render_pass: &mut RenderPass<'b>) {
        self.inner.render(self.wgpu_windowed, render_pass);
    }

    fn render2(&mut self, _: &WgpuBase, encoder: &mut CommandEncoder) {
        self.inner.render2(self.wgpu_windowed, encoder);
    }
}

pub trait WgpuWindowedRender {
    fn render<'a>(&'a mut self, wgpu_windowed: &WgpuWindowed<'_>, render_pass: &mut RenderPass<'a>);
    fn render2(&mut self, wgpu_windowed: &WgpuWindowed<'_>, encoder: &mut CommandEncoder) {}
}

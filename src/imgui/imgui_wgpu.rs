use imgui::Ui;
use imgui_wgpu::Renderer;
use wgpu::RenderPass;
use winit::window::Window;

use super::Imgui;
use crate::wgpu::WgpuWindowed;

pub struct ImguiWgpu<'a> {
    base: Imgui<'a>,
    wgpu_window: &'a WgpuWindowed<'a>,
    renderer: Renderer,
}

impl<'a> ImguiWgpu<'a> {
    pub fn new(window: &'a Window, wgpu_window: &'a WgpuWindowed<'_>) -> Self {
        let mut base = Imgui::new(window);

        let format = wgpu_window.swap_chain_desc.format;

        let mut config = if format.describe().srgb {
            imgui_wgpu::RendererConfig::new_srgb()
        } else {
            imgui_wgpu::RendererConfig::new()
        };
        config.texture_format = format;

        let renderer = imgui_wgpu::Renderer::new(
            base.context.get().unwrap(),
            &wgpu_window.base.device,
            &wgpu_window.base.queue,
            config,
        );

        Self {
            base,
            wgpu_window,
            renderer,
        }
    }

    pub fn render<'b, T>(&'b mut self, renderpass: &mut RenderPass<'b>, func: T)
    where
        T: FnOnce(&mut Ui<'_>),
    {
        if let Some(draw_data) = self.base.render(func) {
            self.renderer
                .render(
                    draw_data,
                    &self.wgpu_window.base.queue,
                    &self.wgpu_window.base.device,
                    renderpass,
                )
                .unwrap();
        }
    }
}

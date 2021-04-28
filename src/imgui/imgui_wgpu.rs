use imgui::{DrawData, Ui};
use imgui_wgpu::Renderer;
use wgpu::RenderPass;
use winit::window::Window;

use super::Imgui;
use crate::wgpu::{wgpu_windowed::WgpuWindowedRender, WgpuWindowed};

pub struct ImguiWgpu<'a> {
    pub base: Imgui<'a>,
    renderer: Renderer,
}

impl<'a> ImguiWgpu<'a> {
    pub fn new(window: &'a Window, wgpu_window: &WgpuWindowed<'_>) -> Self {
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

        Self { base, renderer }
    }

    fn render_impl<'r>(
        renderer: &'r mut Renderer,
        wgpu_window: &WgpuWindowed<'_>,
        renderpass: &mut RenderPass<'r>,
        draw_data: Option<&DrawData>,
    ) {
        if let Some(draw_data) = draw_data {
            renderer
                .render(
                    draw_data,
                    &wgpu_window.base.queue,
                    &wgpu_window.base.device,
                    renderpass,
                )
                .unwrap();
        }
    }

    pub fn render<'r, T>(
        &'r mut self,
        wgpu_windowed: &WgpuWindowed<'_>,
        render_pass: &mut RenderPass<'r>,
        target: &'r mut T,
    ) where
        T: WgpuWindowedRender + ImguiWgpuRender,
    {
        // draw_data borrows self.base, and render_impl borrows self.renderer, so the mut borrow needs to be split
        let ImguiWgpu { base, renderer } = self;

        let draw_data = base.render(|ui| target.render_ui(ui));

        target.render(wgpu_windowed, render_pass);

        Self::render_impl(renderer, wgpu_windowed, render_pass, draw_data);
    }

    pub fn partial_render<'r, T>(
        &'r mut self,
        target: &'r mut T,
    ) -> impl WgpuWindowedRender + 'a + 'r
    where
        T: WgpuWindowedRender + ImguiWgpuRender,
    {
        ImguiWgpuWrapper {
            imgui: self,
            inner: target,
        };
    }
}

struct ImguiWgpuWrapper<'a, 'r, T> {
    imgui: &'r mut ImguiWgpu<'a>,
    inner: &'r mut T,
}

impl<'a, 'b, T> WgpuWindowedRender for ImguiWgpuWrapper<'a, 'b, T>
where
    T: WgpuWindowedRender + ImguiWgpuRender,
{
    fn render<'r>(
        &'r mut self,
        wgpu_windowed: &WgpuWindowed<'_>,
        render_pass: &mut RenderPass<'r>,
    ) {
        self.imgui.render(wgpu_windowed, render_pass, self.inner);
    }
}

pub trait ImguiWgpuRender {
    fn render_ui(&mut self, _: &mut Ui<'_>);
}

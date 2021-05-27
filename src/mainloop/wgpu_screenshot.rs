use std::time::Duration;

use pollster::FutureExt as _;
use wgpu::{
    Buffer, BufferDescriptor, BufferUsage, CommandEncoder, ImageCopyBuffer, ImageCopyTexture,
    RenderPass, Texture, TextureDescriptor, TextureUsage,
};
use winit::{
    event::{Event, VirtualKeyCode},
    window::Window,
};

use crate::imgui::ImguiWgpuRender;
use crate::util::{
    texture_image_layout, texture_size, to_image, CreateFromWgpu, TextureResult, WindowSize,
};
use crate::wgpu::{WgpuBase, WgpuBaseRender, WgpuWindowed, WgpuWindowedRender};

use super::{Mainloop, WgpuImguiWindowMainloop};

pub struct WgpuScreenshot<'a, T> {
    should_screenshot: bool,
    inner: WgpuImguiWindowMainloop<'a, T>,
}

impl<'a, T> WgpuScreenshot<'a, T>
where
    T: CreateFromWgpu,
{
    pub fn new(window: &'a Window) -> Self {
        Self {
            should_screenshot: false,
            inner: WgpuImguiWindowMainloop::new(window),
        }
    }
}

impl<T> Mainloop for WgpuScreenshot<'_, T>
where
    T: WgpuWindowedRender + ImguiWgpuRender,
{
    fn event(&mut self, event: &Event<'_, ()>) {
        self.inner.event(event)
    }

    fn keyboard(&mut self, key: VirtualKeyCode) {
        self.inner.keyboard(key);

        match key {
            VirtualKeyCode::F1 => self.should_screenshot = true,
            _ => {}
        }
    }

    fn update(&mut self, delta: Duration) {
        self.inner.update(delta)
    }

    fn render(&mut self) {
        if !self.should_screenshot {
            return self.inner.render();
        }
        self.should_screenshot = false;

        println!("screenshot");

        let WgpuImguiWindowMainloop {
            wgpu_window,
            imgui,
            state,
        } = &mut self.inner;

        let base = &wgpu_window.base;

        let desc_orig = wgpu_window.desc().aligned();
        dbg!(desc_orig.width);
        let desc = desc_orig.into_2d(
            TextureUsage::COPY_SRC | TextureUsage::SAMPLED | TextureUsage::RENDER_ATTACHMENT,
        );
        let TextureResult { texture, view, .. } = base.texture(&desc, Default::default(), None);

        let buffer = base.device.create_buffer(&BufferDescriptor {
            size: texture_size(&desc) as _,
            usage: BufferUsage::COPY_DST | BufferUsage::MAP_READ,
            label: None,
            mapped_at_creation: false,
        });

        let mut to_render = imgui.partial_render(state); // todo: deduplicate WgpuImguiWindowMainloop.render()
        let mut renderer = ScreenshotRender {
            inner: &mut to_render,
            wgpu_window,
            texture: &texture,
            buffer: &buffer,
            desc: &desc,
        };

        base.render(&view, &mut renderer);

        let slice = buffer.slice(..);

        // todo: move to new thread, or poll(Maintain::Poll) single threaded across multiple frames?
        let mapping = slice.map_async(wgpu::MapMode::Read);
        base.device.poll(wgpu::Maintain::Wait);
        mapping.block_on().unwrap();

        let data = slice.get_mapped_range();
        let image = to_image(&data, &desc_orig).unwrap().into_rgba8();
        image.save("out.png").unwrap();
    }

    fn resize(&mut self, size: WindowSize) {
        self.inner.resize(size)
    }

    fn ignore_keyboard(&self) -> bool {
        self.inner.ignore_keyboard()
    }
}

struct ScreenshotRender<'a, T> {
    inner: &'a mut T,
    wgpu_window: &'a WgpuWindowed<'a>,
    texture: &'a Texture,
    buffer: &'a Buffer,
    desc: &'a TextureDescriptor<'a>,
}

impl<T> WgpuBaseRender for ScreenshotRender<'_, T>
where
    T: WgpuWindowedRender,
{
    fn render<'a>(&'a mut self, _: &WgpuBase, render_pass: &mut RenderPass<'a>) {
        self.inner.render(self.wgpu_window, render_pass);
    }

    fn render_encoder(&mut self, wgpu_base: &WgpuBase, encoder: &mut CommandEncoder, after: bool) {
        self.inner.render_encoder(self.wgpu_window, encoder, after);

        if !after {
            return;
        }

        // todo: fragment shader to copy texture to swapchain texture?
        // imgui is stretched during screenshot because view size != expected resize()
        // todo: create one renderpipeline to cover all "copy texture w/ fullscreen tri" needs
        // todo: screenshot pipeline (N is normal size, A is 256 aligned size, S is swapchain) (render pipeline) -> N; N -> S; N -> A (no stretch); A buffer etc
        // ^ can A be png color format? convert in gpu, not image crate

        encoder.copy_texture_to_buffer(
            ImageCopyTexture {
                texture: self.texture,
                mip_level: Default::default(),
                origin: Default::default(),
            },
            ImageCopyBuffer {
                buffer: self.buffer,
                layout: texture_image_layout(self.desc),
            },
            self.desc.size,
        )
    }
}

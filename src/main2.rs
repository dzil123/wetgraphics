use std::{iter, marker::PhantomData, path::Path};

use imgui::DrawData;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window as _, WindowBuilder},
};

use pollster::FutureExt as _;

type Event<'a> = winit::event::Event<'a, ()>;

type WindowSize = winit::dpi::PhysicalSize<u32>;

struct Window {
    event_loop: winit::event_loop::EventLoop<()>,
    window: winit::window::Window,
}

impl Window {
    fn new() -> Self {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new().build(&event_loop).unwrap();

        Self { event_loop, window }
    }

    fn run<T>(self, mut mainloop: T) -> !
    where
        T: Mainloop + 'static,
    {
        let Self { event_loop, window } = self;

        event_loop.run(move |event: Event, _, control_flow| {
            mainloop.event(&event);

            match event {
                Event::RedrawRequested(_) => {
                    // let should_close = mainloop.render();
                    // if should_close {
                    //     *control_flow = ControlFlow::Exit
                    // }
                    mainloop.render();
                }
                Event::MainEventsCleared => {
                    window.request_redraw();
                }
                Event::WindowEvent { event, window_id } if window_id == window.id() => {
                    match event {
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        WindowEvent::Resized(size)
                        | WindowEvent::ScaleFactorChanged {
                            new_inner_size: &mut size,
                            ..
                        } => mainloop.resize(size),
                        WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    virtual_keycode: Some(key),
                                    state: ElementState::Pressed,
                                    ..
                                },
                            ..
                        } if !mainloop.ignore_keyboard() => match key {
                            VirtualKeyCode::Escape | VirtualKeyCode::Q => {
                                *control_flow = ControlFlow::Exit
                            }
                            _ => {}
                        },
                        _ => {}
                    }
                }
                _ => {}
            }
        })
    }
}

impl SafeWgpuSurface for Window {
    fn create_surface(&self, instance: &wgpu::Instance) -> wgpu::Surface {
        // SAFETY: this is always safe for a valid winit::Window
        unsafe { instance.create_surface(&self.window) }
    }
}

trait Mainloop {
    fn event(&mut self, _event: &Event) {}
    fn input(&mut self, _event: &WindowEvent) {}
    fn render(&mut self) {}
    // fn render(&mut self) -> bool {
    //     false
    // }

    fn resize(&mut self, _size: WindowSize) {}

    fn ignore_keyboard(&self) -> bool {
        false
    }

    fn ignore_mouse(&self) -> bool {
        false
    }
}

struct ImguiWgpu<'a> {
    base: Imgui<'a>,
    wgpu_window: &'a WgpuWindowed,
    renderer: imgui_wgpu::Renderer,
}

impl<'a> ImguiWgpu<'a> {
    fn new(wgpu_window: &'a WgpuWindowed, window: &'a winit::window::Window) -> Self {
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

    fn render(&mut self) {
        let frame: ImguiFrame = None.unwrap();
        let mut renderpass: wgpu::RenderPass = None.unwrap();

        self.renderer
            .render(
                frame.render(),
                &self.wgpu_window.base.queue,
                &self.wgpu_window.base.device,
                &mut renderpass,
            )
            .unwrap();
    }
}

struct ImguiMainloop<'a> {
    imgui: Imgui<'a>,
}

impl<'a> Mainloop for ImguiMainloop<'a> {
    fn event(&mut self, event: &Event) {
        self.imgui.event(event);
    }
}

struct App<'a> {
    wgpu: WgpuWindowed,
    imgui: ImguiWgpu<'a>,
}

impl<'a> Mainloop for App<'a> {}

pub fn main() {
    wgpu_subscriber::initialize_default_subscriber(None);

    let window = Window::new();
    let wgpu = WgpuWindowed::new(&window);
    let imgui = ImguiWgpu::new(&wgpu, &window.window);

    // let app = App { wgpu, imgui };
}

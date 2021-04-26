use std::{iter, marker::PhantomData, path::Path};

use imgui::DrawData;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window as _, WindowBuilder},
};

use pollster::FutureExt as _;

// simple render pass that only clears the frame to black. ignore if using depth buffer, not clearing frame, or anything more complex
fn begin_render_pass<'a>(
    encoder: &'a mut wgpu::CommandEncoder,
    texture: &'a wgpu::TextureView,
) -> wgpu::RenderPass<'a> {
    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
            attachment: texture,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                ..Default::default()
            },
        }],
        ..Default::default()
    })
}

type Event<'a> = winit::event::Event<'a, ()>;

struct WgpuBase {
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: Option<wgpu::Surface>,
}

impl WgpuBase {
    fn new<T>(window: Option<&T>) -> Self
    where
        T: SafeWgpuSurface,
    {
        let instance = wgpu::Instance::new(wgpu::BackendBit::VULKAN);

        let surface = window.map(|window| window.create_surface(&instance));

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: surface.as_ref(),
                ..Default::default()
            })
            .block_on()
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::PUSH_CONSTANTS,
                    limits: wgpu::Limits {
                        max_push_constant_size: 128, // i have it on good authority that this is the max for a rx 580 and igpu
                        ..Default::default()
                    },
                    ..Default::default()
                },
                None,
            )
            .block_on()
            .unwrap();

        Self {
            adapter,
            device,
            queue,
            surface,
        }
    }

    fn render<T>(&self, texture: &wgpu::TextureView, mut f: T)
    where
        T: FnMut(wgpu::RenderPass),
    {
        let mut encoder = self.device.create_command_encoder(&Default::default());

        f(begin_render_pass(&mut encoder, texture));

        self.queue.submit(iter::once(encoder.finish()));
    }
}

// should this store window?
struct WgpuWindowed {
    base: WgpuBase,
    surface: wgpu::Surface,
    swap_chain_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
}

impl WgpuWindowed {
    fn new(window: &Window) -> Self {
        let mut base = WgpuBase::new(Some(window));
        let surface = base.surface.take().unwrap();

        let size = window.window.inner_size();

        let swap_chain_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8Unorm,
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
        }
    }

    fn resize(&mut self, size: Option<winit::dpi::PhysicalSize<u32>>) {
        if let Some(size) = size {
            self.swap_chain_desc.width = size.width;
            self.swap_chain_desc.height = size.height;
        }

        self.swap_chain = self
            .base
            .device
            .create_swap_chain(&self.surface, &self.swap_chain_desc);
    }

    fn render<T>(&mut self, f: T)
    where
        T: FnMut(wgpu::RenderPass),
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

        self.base.render(&texture.view, f);
    }
}

trait SafeWgpuSurface {
    fn create_surface(&self, instance: &wgpu::Instance) -> wgpu::Surface;
}

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

enum ImguiStatus {
    Enabled(imgui::Context),
    Suspended(imgui::SuspendedContext),
}

impl ImguiStatus {
    fn get(&mut self) -> Option<&mut imgui::Context> {
        if let ImguiStatus::Enabled(context) = self {
            Some(context)
        } else {
            None
        }
    }
}

// Imgui for winit
struct Imgui<'a> {
    context: ImguiStatus,
    platform: imgui_winit_support::WinitPlatform,
    window: &'a winit::window::Window,
}

#[must_use]
struct ImguiFrame<'a, 'ui> {
    ui: imgui::Ui<'ui>,
    platform: &'a mut imgui_winit_support::WinitPlatform,
    window: &'a winit::window::Window,
}

impl<'a, 'ui> ImguiFrame<'a, 'ui> {
    #[must_use]
    fn render(self) -> &'ui imgui::DrawData {
        self.platform.prepare_render(&self.ui, self.window);
        self.ui.render()
    }
}

impl<'a> Imgui<'a> {
    fn new(window: &'a winit::window::Window) -> Self {
        use imgui::{FontConfig, FontSource};
        use imgui_winit_support::{HiDpiMode, WinitPlatform};

        let mut context = imgui::Context::create();
        context.set_ini_filename(None);

        let mut platform = WinitPlatform::init(&mut context);
        platform.attach_window(context.io_mut(), &window, HiDpiMode::Rounded);

        let hidpi_factor = platform.hidpi_factor();
        let font_size = (13.0 * hidpi_factor) as f32;
        context.fonts().add_font(&[FontSource::DefaultFontData {
            config: Some(FontConfig {
                size_pixels: font_size,
                ..Default::default()
            }),
        }]);

        context.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;

        Self {
            context: ImguiStatus::Enabled(context),
            platform,
            window,
        }
    }

    fn event(&mut self, event: &Event) -> Option<()> {
        self.platform
            .handle_event(self.context.get()?.io_mut(), self.window, event);
        Some(())
    }

    #[must_use]
    fn draw(&mut self) -> Option<ImguiFrame> {
        let context = self.context.get()?;

        self.platform
            .prepare_frame(context.io_mut(), self.window)
            .unwrap();

        Some(ImguiFrame {
            ui: context.frame(),
            window: self.window,
            platform: &mut self.platform,
        })
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

use std::{iter, marker::PhantomData, path::Path};

use imgui::DrawData;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window as _, WindowBuilder},
};

use pollster::FutureExt as _;

struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
}

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

impl State {
    fn new(window: &winit::window::Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::BackendBit::VULKAN);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .block_on()
            .unwrap();

        let pushconst_device = wgpu::DeviceDescriptor {
            features: wgpu::Features::PUSH_CONSTANTS,
            limits: wgpu::Limits {
                max_push_constant_size: 128, // i have it on good authority that this is the max for a rx 580 and igpu
                ..Default::default()
            },
            ..Default::default()
        };

        let (device, queue) = adapter
            .request_device(&pushconst_device, None)
            .block_on()
            .unwrap();

        dbg!(device.features(), device.limits());

        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        let vs_module = crate::shaders::load(&device, "shader.vert");
        let fs_module = crate::shaders::load(&device, "shader.frag");

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&device.create_pipeline_layout(&Default::default())),
            vertex: wgpu::VertexState {
                module: &vs_module,
                entry_point: "main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &fs_module,
                entry_point: "main",
                targets: &[wgpu::ColorTargetState {
                    format: sc_desc.format,
                    alpha_blend: wgpu::BlendState::REPLACE,
                    color_blend: wgpu::BlendState::REPLACE,
                    write_mask: wgpu::ColorWrite::ALL,
                }],
            }),
            primitive: wgpu::PrimitiveState {
                cull_mode: wgpu::CullMode::Back,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: Default::default(),
            // multisample: wgpu::MultisampleState {
            //     count: 2,
            //     ..Default::default()
            // },
        });

        Self {
            surface,
            device,
            queue,
            size,
            sc_desc,
            swap_chain,
            render_pipeline,
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        // println!("resize");
        self.size = new_size;
        self.sc_desc.width = new_size.width;
        self.sc_desc.height = new_size.height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
    }

    #[allow(unused_variables)]
    fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }

    fn update(&mut self) {}

    fn render(&mut self) -> Result<(), wgpu::SwapChainError> {
        // let texture = self.swap_chain.get_current_frame()?;
        // if texture.suboptimal {
        //     println!("suboptimal");
        // }
        // let frame = texture.output;
        let frame = self.swap_chain.get_current_frame()?.output;

        let mut encoder = self.device.create_command_encoder(&Default::default());

        {
            let mut render_pass = begin_render_pass(&mut encoder, &frame.view);
            // TODO make this into a closure that only takes in render_pass

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.draw(0..3, 0..1);
        }

        self.queue.submit(iter::once(encoder.finish()));

        Ok(())
    }
}

type Event<'a> = winit::event::Event<'a, ()>;

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

    fn create_surface(&self, instance: wgpu::Instance) -> wgpu::Surface {
        // SAFETY: this is always safe for a valid winit::Window
        unsafe { instance.create_surface(&self.window) }
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
                    mainloop.render();
                }
                Event::MainEventsCleared => {
                    window.request_redraw();
                }
                Event::WindowEvent { event, window_id } if window_id == window.id() => {
                    match event {
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
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

trait Mainloop {
    fn event(&mut self, _event: &Event) {}
    fn input(&mut self, _event: &WindowEvent) {}
    fn render(&mut self) {}

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

fn foo() {
    let mut x = Imgui::new(None.unwrap());
    let frame = x.draw().unwrap();

    let ui = &frame.ui;
    ui.text("foobar");

    let _ = frame.render();
}

struct ImguiMainloop<'a> {
    imgui: Imgui<'a>,
}

impl<'a> Mainloop for ImguiMainloop<'a> {
    fn event(&mut self, event: &Event) {
        self.imgui.event(event);
    }
}

fn compile_glsl(filename: &str) {
    // wgpu::ShaderModuleDescriptor
    // wgpu::util::make_spirv
}

pub fn main() {
    // let x: Result<&'static str, &'static str> = Err("this is a triumph");
    // x.unwrap();
    // panic!("this is a triumph");

    // foo();
    wgpu_subscriber::initialize_default_subscriber(None);

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut state = State::new(&window);

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                if state.input(event) {
                    return;
                }
                match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::KeyboardInput { input, .. } => match input {
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        } => *control_flow = ControlFlow::Exit,
                        _ => {}
                    },
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        // new_inner_size is &mut so w have to dereference it twice
                        state.resize(**new_inner_size);
                    }
                    // &WindowEvent::Resized(size)
                    // | &WindowEvent::ScaleFactorChanged {
                    //     new_inner_size: &mut size,
                    //     ..
                    // } => state.resize(size),
                    _ => {}
                }
            }
            Event::RedrawRequested(_) => {
                state.update();
                match state.render() {
                    // Recreate the swap_chain if lost
                    Err(wgpu::SwapChainError::Lost) => state.resize(state.size),
                    // Err(wgpu::SwapChainError::Outdated) => state.resize(window.inner_size()),
                    // The system is out of memory, we should probably quit
                    other => other.unwrap(),
                }
            }
            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                window.request_redraw();
            }
            _ => {}
        }
    });
}

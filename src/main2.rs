use std::{iter, marker::PhantomData};

use imgui::DrawData;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window as _, WindowBuilder},
};

struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
}

impl State {
    async fn new(window: &winit::window::Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::BackendBit::VULKAN);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        let vs_module = device.create_shader_module(&wgpu::include_spirv!("shader.vert.spv"));
        let fs_module = device.create_shader_module(&wgpu::include_spirv!("shader.frag.spv"));

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
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
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::Back,
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
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
        let frame = self.swap_chain.get_current_frame()?.output;

        let mut encoder = self.device.create_command_encoder(&Default::default());

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                ..Default::default()
            });

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

// Imgui for winit
struct Imgui<'a> {
    context: imgui::Context,
    platform: imgui_winit_support::WinitPlatform,
    window: &'a winit::window::Window,
}

#[must_use = "foo"]
struct ImguiFrame<'a, 'ui> {
    ui: imgui::Ui<'ui>,
    imgui: &'a mut Imgui<'a>,
    // window: &'a winit::window::Window,
    // platform: &'a mut imgui_winit_support::WinitPlatform,
}

// impl<'a, 'ui> ImguiFrame<'a, 'ui> {
//     fn render2(self) -> &'ui imgui::DrawData {
//         // let ImguiFrame(ui) = frame;
//         let ui = None.unwrap();
//         self.platform.prepare_render(&ui, self.window);
//         // let draw_data = ui.render();
//         ui.render()
//     }
// }

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
            context,
            platform,
            window,
        }
    }

    fn event(&mut self, event: &Event) {
        self.platform
            .handle_event(self.context.io_mut(), self.window, event)
    }

    fn render(&'a mut self) -> ImguiFrame<'a, '_> {
        self.platform
            .prepare_frame(self.context.io_mut(), self.window)
            .unwrap();
        let ui = self.context.frame();
        ImguiFrame { ui, imgui: self }
        // ImguiFrame {
        //     ui,
        //     window: self.window,
        //     platform: &mut self.platform,
        // }
    }

    fn render2(&mut self, frame: ImguiFrame) {
        // let ImguiFrame(ui) = frame;
        let ui = None.unwrap();
        self.platform.prepare_render(&ui, self.window);
        let draw_data = ui.render();
    }
}

fn foo() {
    let mut x: Imgui = None.unwrap();
    let frame = x.render();

    {
        // let ref ui = frame.0;
        let ui: imgui::Ui<'_> = None.unwrap();
        ui.text("foobar");
    }

    x.render2(frame);
}

struct ImguiMainloop<'a> {
    imgui: Imgui<'a>,
}

impl<'a> Mainloop for ImguiMainloop<'a> {
    fn event(&mut self, event: &Event) {
        self.imgui.event(event)
    }
}

pub fn main() {
    foo();
    wgpu_subscriber::initialize_default_subscriber(None);

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    use futures::executor::block_on;

    // Since main can't be async, we're going to need to block
    let mut state = block_on(State::new(&window));

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
                    _ => {}
                }
            }
            Event::RedrawRequested(_) => {
                state.update();
                match state.render() {
                    // Recreate the swap_chain if lost
                    Err(wgpu::SwapChainError::Lost) => state.resize(state.size),
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

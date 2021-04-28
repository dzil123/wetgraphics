use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    platform::run_return::EventLoopExtRunReturn as _,
    window::{Window as WinitWindow, WindowBuilder},
};

use crate::{mainloop::Mainloop, util::SafeWgpuSurface};

pub struct Window {
    event_loop: EventLoop<()>,
}

impl Window {
    pub fn new() -> (Self, WinitWindow) {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new().build(&event_loop).unwrap();

        (Self { event_loop }, window)
    }

    // although this function returns, you should not do anything after this function
    // this is only using event_loop.run_return() instead of event_loop.run() to workaround the T: 'static requirement otherwise
    pub fn run<T>(mut self, window: &WinitWindow, mut mainloop: T)
    where
        T: Mainloop,
    {
        self.event_loop.run_return(move |event, _, control_flow| {
            mainloop.event(&event);

            match event {
                Event::RedrawRequested(window_id) if window_id == window.id() => {
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
                    mainloop.input(&event); // todo find way to apply ignore_keyboard() to this
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
                Event::RedrawRequested(_) | Event::WindowEvent { .. } => {
                    panic!("invalid window_id {:#?}", event)
                }
                _ => {}
            }
        })
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        println!("exited cleanly");
    }
}

impl SafeWgpuSurface for WinitWindow {
    fn create_surface(&self, instance: &wgpu::Instance) -> wgpu::Surface {
        // SAFETY: this is always safe for a valid winit::Window,
        // as long as the winit::Window is alive for as long as the wgpu surface/swapchain exists
        // https://github.com/gfx-rs/wgpu-rs/issues/674
        unsafe { instance.create_surface(self) }
    }
}

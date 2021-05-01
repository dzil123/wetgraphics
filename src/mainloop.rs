use crate::util::WindowSize;
use winit::event::{Event, WindowEvent};

pub trait Mainloop {
    fn event(&mut self, _event: &Event<'_, ()>) {}
    fn input(&mut self, _event: &WindowEvent<'_>) {}
    fn render(&mut self) {}
    // fn render(&mut self) -> bool {
    //     false
    // }

    fn resize(&mut self, _size: WindowSize) {}

    fn ignore_keyboard(&self) -> bool {
        false
    }
}

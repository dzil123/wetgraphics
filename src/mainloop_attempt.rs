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

trait Mainloop {
    type Inner: Mainloop;
    type RenderParams;

    fn event(&mut self, event: &Event) {
        self.inner_mut().event(event)
    }

    fn render(&mut self, params: Self::RenderParams);

    fn ignore_keyboard(&self) -> bool {
        self.inner().ignore_keyboard()
    }

    fn inner(&self) -> &Self::Inner;
    fn inner_mut(&mut self) -> &mut Self::Inner;
}

impl Mainloop for () {
    type Inner = ();
    type RenderParams = ();

    fn event(&mut self, _: &Event) {}

    fn render(&mut self, _: Self::RenderParams) {}

    fn ignore_keyboard(&self) -> bool {
        false
    }

    fn inner(&self) -> &Self::Inner {
        self
    }

    fn inner_mut(&mut self) -> &mut Self::Inner {
        self
    }
}

fn run_mainloop_render<T>(mainloop: &mut T)
where
    T: Mainloop<RenderParams = ()>,
{
    mainloop.render(())
}

trait ImguiApp {
    fn render(&mut self, context: &mut imgui::Ui);
}

struct Imgui<I, O> {
    inner: I,
    _outer: PhantomData<*const O>,
}

impl<I: Mainloop, O: 'static + ImguiApp> Mainloop for Imgui<I, O> {
    type Inner = I;

    type RenderParams = (&'static mut O, u32);

    fn render(&mut self, params: Self::RenderParams) {
        let mut ui: imgui::Ui = todo!();
        ImguiApp::render(params.0, &mut ui);
        let _ = ui.render();
    }

    fn inner(&self) -> &Self::Inner {
        &self.inner
    }

    fn inner_mut(&mut self) -> &mut Self::Inner {
        &mut self.inner
    }
}

pub fn main() {
    let mut x = ();

    x.inner_mut().render(());

    run_mainloop_render(&mut ())
}

// chang da wolrd
// my fnial mesaeg

use imgui::{Context, DrawData, FontConfig, FontSource, SuspendedContext, Ui};
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use winit::{event::Event, window::Window};

pub enum ImguiStatus {
    Enabled(Context),
    Suspended(SuspendedContext),
}

impl ImguiStatus {
    pub fn get(&mut self) -> Option<&mut Context> {
        if let ImguiStatus::Enabled(context) = self {
            Some(context)
        } else {
            None
        }
    }
}

// Imgui for winit
pub struct Imgui<'a> {
    pub context: ImguiStatus,
    pub platform: WinitPlatform,
    pub window: &'a Window,
}

impl<'a> Imgui<'a> {
    pub fn new(window: &'a Window) -> Self {
        let mut context = Context::create();
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

    fn event(&mut self, event: &Event<'_, ()>) -> Option<()> {
        self.platform
            .handle_event(self.context.get()?.io_mut(), self.window, event);
        Some(())
    }

    pub fn render<T>(&mut self, func: T) -> Option<&DrawData>
    where
        T: FnOnce(&mut Ui<'_>),
    {
        let context = self.context.get()?;

        self.platform
            .prepare_frame(context.io_mut(), self.window)
            .unwrap();

        let mut ui = context.frame();
        func(&mut ui);

        self.platform.prepare_render(&ui, self.window);
        let draw_data = ui.render();

        Some(draw_data)
    }
}

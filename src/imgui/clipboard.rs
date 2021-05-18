use clipboard::{ClipboardContext, ClipboardProvider};
use imgui::{ClipboardBackend, ImStr, ImString};

pub struct ClipboardSupport(ClipboardContext);

pub fn init() -> Option<ClipboardSupport> {
    ClipboardContext::new().map_or_else(
        |err| {
            dbg!(err);
            None
        },
        |ctx| Some(ClipboardSupport(ctx)),
    )
}

impl ClipboardBackend for ClipboardSupport {
    fn get(&mut self) -> Option<ImString> {
        // self.0.get_contents().ok().map(|text| text.into())
        self.0.get_contents().map_or_else(
            |err| {
                dbg!(err);
                None
            },
            |t| Some(t.into()),
        )
    }
    fn set(&mut self, text: &ImStr) {
        // let _ = self.0.set_contents(text.to_str().to_owned());
        if let Err(err) = self.0.set_contents(text.to_str().to_owned()) {
            dbg!(err);
        }
    }
}

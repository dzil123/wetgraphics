pub trait Mainloop {
    type Inner: Mainloop;
    type RenderParams;

    fn render(&mut self, params: Self::RenderParams);
}

impl Mainloop for () {
    type Inner = ();
    type RenderParams = ();

    fn render(&mut self, _: Self::RenderParams) {}
}

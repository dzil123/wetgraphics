use fxhash::FxHashMap;
use wgpu::ShaderModule;

use super::WgpuBase;

pub(super) type Shaders = FxHashMap<&'static str, ShaderModule>;

impl WgpuBase {
    // this arrangement is needed because returning a &T from a &mut self method
    // makes the return value borrow over &mut self, which makes it impossible to
    // use &self while the return value is still held

    pub fn shader_preload(&mut self, name: &'static str) {
        let device = &self.device;
        self.shaders
            .entry(name)
            .or_insert_with(|| crate::shaders::load(device, name));
    }

    pub fn shader(&self, name: &'static str) -> &ShaderModule {
        self.shaders
            .get(name)
            .expect("shader not loaded, run preload first")
    }
}

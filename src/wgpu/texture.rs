use wgpu::{util::DeviceExt, Texture, TextureDescriptor, TextureView};

use crate::util::{texture_size, InitType};

use super::WgpuBase;

impl WgpuBase {
    pub fn texture(&self, desc: &TextureDescriptor<'static>, init: InitType<'_>) -> TextureResult {
        let texture = init.create(
            texture_size(desc),
            || self.device.create_texture(desc),
            |data| {
                self.device
                    .create_texture_with_data(&self.queue, desc, data)
            },
        );
        let view = texture.create_view(&Default::default());

        TextureResult {
            texture,
            view,
            desc: desc.clone(),
        }
    }
}

// wgpu does implicit Arc<> semantics in the background, so eg texture can be dropped but view will remain valid
pub struct TextureResult {
    pub texture: Texture,
    pub view: TextureView,
    pub desc: TextureDescriptor<'static>,
}

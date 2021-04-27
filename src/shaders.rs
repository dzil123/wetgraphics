type Shader = &'static [u8];
type ShaderResult = Result<Shader, &'static str>;

// static SHADERS: phf::Map<&'static str, ShaderResult>
include!(concat!(env!("OUT_DIR"), "/codegen_shaders.rs"));

fn lookup(name: &str) -> ShaderResult {
    *SHADERS.get(name).unwrap_or(&Err("Not Found"))
}

#[track_caller]
pub fn load(device: &wgpu::Device, name: &str) -> wgpu::ShaderModule {
    let shader = match lookup(name) {
        Ok(shader) => shader,
        Err(err) => panic!("Could not load shader '{}': {}", name, err),
    };

    let shader = wgpu::ShaderModuleDescriptor {
        label: Some(name),
        source: wgpu::util::make_spirv(shader),
        flags: if cfg!(debug_assertions) {
            wgpu::ShaderFlags::VALIDATION
        } else {
            wgpu::ShaderFlags::empty()
        },
    };

    device.create_shader_module(&shader)
}

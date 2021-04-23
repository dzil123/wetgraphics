use std::borrow::Cow;

type Shader = &'static [u8];
type ShaderResult = Result<Shader, &'static str>;

// static SHADERS: phf::Map<&'static str, ShaderResult>
include!(concat!(env!("OUT_DIR"), "/codegen_shaders.rs"));

fn lookup(name: &str) -> ShaderResult {
    *SHADERS.get(name).unwrap_or(&Err("Not Found"))
}

pub fn load(name: &str) -> wgpu::ShaderModuleDescriptor {
    let shader = match lookup(name) {
        Ok(shader) => shader,
        Err(err) => panic!("Could not load shader '{}' because {}", name, err),
    };

    wgpu::ShaderModuleDescriptor {
        label: Some(name),
        source: wgpu::util::make_spirv(shader),
        flags: wgpu::ShaderFlags::VALIDATION,
    }
}

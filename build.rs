use std::env;
use std::fs::{read_to_string, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use shaderc::{CompileOptions, Compiler, OptimizationLevel, ShaderKind};

#[derive(Debug)]
struct Shader {
    file: PathBuf,
    shader_type: ShaderKind,
    filename: String, // relative to root folder, but can be arbitrary
}

struct Error(String);

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self(format!("IO Error: {}", err))
    }
}

impl From<shaderc::Error> for Error {
    fn from(err: shaderc::Error) -> Self {
        Self(format!("Compile Error: {}", err))
    }
}

fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/shaders");

    // allow utf8 paths everywhere except filename and extention
    let queue: Vec<_> = walkdir::WalkDir::new(&root)
        .follow_links(true)
        .into_iter()
        .map(|entry| entry.unwrap())
        .filter(|entry| entry.file_type().is_file())
        .filter_map(|file| {
            let file = file.into_path();
            let shader_type = match file.extension()?.to_str()? {
                "frag" => ShaderKind::Fragment,
                "vert" => ShaderKind::Vertex,
                "comp" => ShaderKind::Compute,
                _ => return None,
            };
            // let filename = file.file_name()?.to_str()?.to_owned();
            let filename = file.strip_prefix(&root).unwrap().to_str()?.to_owned();

            Some(Shader {
                file,
                shader_type,
                filename,
            })
        })
        .collect();

    // format!("{:#?}", queue)
    //     .lines()
    //     .for_each(|line| println!("cargo:warning={}", line));

    let mut options = CompileOptions::new().unwrap();
    options.set_optimization_level(OptimizationLevel::Zero);
    options.set_warnings_as_errors();
    let options = Some(&options);

    let mut compiler = Compiler::new().unwrap();

    let mut compile = |info: &Shader| -> Result<_, Error> {
        let source = read_to_string(&info.file)?;
        let artifact = compiler.compile_into_spirv(
            &source,
            info.shader_type,
            &info.filename,
            "main",
            options,
        )?;

        Ok(artifact)
    };

    let mut codegen = phf_codegen::Map::<&str>::new();

    for info in queue.iter() {
        let code = match compile(info) {
            Ok(artifact) => format!("Ok(&{:?})", artifact.as_binary_u8()),
            Err(err) => format!("Err({:?})", err.0),
        };

        codegen.entry(&info.filename, &code);
    }

    let path = Path::new(&env::var_os("OUT_DIR").unwrap()).join("codegen_shaders.rs");
    println!("cargo:warning={:?}", path);

    let mut file = BufWriter::new(File::create(&path).unwrap());

    write!(
        &mut file,
        "static SHADERS: phf::Map<&'static str, ShaderResult> = {};",
        codegen.build()
    )
    .unwrap();

    // manually sync bufwriter + file to catch errors instead of during drop
    file.into_inner().unwrap().sync_all().unwrap();
}

use std::env;
use std::fs::{read_to_string, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use lazy_static::lazy_static;
use shaderc::{
    CompileOptions, Compiler, IncludeCallbackResult, IncludeType, OptimizationLevel,
    ResolvedInclude, ShaderKind,
};

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

fn dbg(x: impl std::fmt::Debug) {
    println!("cargo:warning={:?}", x);
}

lazy_static! {
    static ref REPO_ROOT: &'static Path = Path::new(env!("CARGO_MANIFEST_DIR"));
    static ref ROOT: PathBuf = REPO_ROOT.join("src/shaders");
    static ref STD_ROOT: PathBuf = ROOT.join("std");
    static ref LYGIA_ROOT: PathBuf = REPO_ROOT.join("lygia");
}

// since this takes &str, it probably isnt OsStr safe
fn include_lygia(
    include: &str,
    ty: IncludeType,
    source: &str,
    depth: usize,
) -> IncludeCallbackResult {
    assert!(depth < 5);

    let source = Path::new(source);
    let include = Path::new(include);

    assert!(include.is_relative());

    let path = match ty {
        IncludeType::Standard => {
            assert!(source.is_relative()); // implicit prefix of ROOT

            let root = if include.starts_with("lygia/") {
                LYGIA_ROOT.parent().unwrap()
            } else {
                &*STD_ROOT
                // return Err(format!("invalid include: {}", include.display()));
            };

            root.join(include)
        }
        IncludeType::Relative => {
            assert!(source.is_absolute()); // this is a sub dependency of a lygia file, comes from resolved_name

            source.parent().unwrap().join(include)
        }
    };

    Ok(ResolvedInclude {
        resolved_name: path.to_string_lossy().into(),
        content: read_to_string(&path).map_err(|err| Error::from(err).0)?,
    })
}

fn main() {
    // allow utf8 paths everywhere except filename and extention
    let queue: Vec<_> = walkdir::WalkDir::new(&*ROOT)
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
            let filename = file.strip_prefix(&*ROOT).unwrap().to_str()?.to_owned();

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
    options.set_include_callback(include_lygia);
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
    dbg(&path);

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

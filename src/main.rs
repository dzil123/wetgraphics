#[cfg(feature = "shader_compile")]
mod shader_compile;

#[cfg(not(feature = "shader_compile"))]
mod main2;

fn main() {
    #[cfg(feature = "shader_compile")]
    shader_compile::main().unwrap();

    #[cfg(not(feature = "shader_compile"))]
    main2::main();
}

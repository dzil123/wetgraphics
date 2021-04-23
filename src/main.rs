#![allow(unused_imports, unused_variables, unreachable_code, dead_code)]

mod main2;

mod shaders;

fn main() {
    #[cfg(not(feature = "shader_compile"))]
    main2::main();
}

use arduino_build_helpers::{arduino_include_root, ArduinoBindgen, ArduinoBuilder};
use std::env;
use std::path::PathBuf;

fn arducam_git_src() -> &'static str {
    "../submodules/ArduCAM/ArduCAM"
}

fn wire_include_dir() -> String {
    format!("{}/libraries/Wire/src", arduino_include_root())
}

fn spi_include_dir() -> String {
    format!("{}/libraries/SPI/src", arduino_include_root())
}

fn generate_bindings_generic(header: String, out_basename: &str) {
    println!("cargo:rerun-if-changed={}", header);
    let bindings = bindgen::Builder::default()
        .header(header)
        .rig_arduino_uno()
        .clang_args(&[
            &format!("-I{}", arducam_git_src()),
            &format!("-I{}", spi_include_dir()),
            &format!("-I{}", wire_include_dir()),
            "-x",
            "c++",
        ])
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        // .ctypes_prefix("rust_arduino_runtime::workaround_cty")
        .ctypes_prefix("cty") // using this causes `undefined reference` link errors
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let bindings_out_file = out_path.join(out_basename);
    bindings
        .write_to_file(bindings_out_file)
        .expect("Couldn't write bindings!");
}

fn generate_bindings_rs() {
    generate_bindings_generic(format!("{}/ArduCAM.h", arducam_git_src()), "bindings.rs");
    generate_bindings_generic(
        format!("{}/libraries/Wire/src/Wire.h", arduino_include_root()),
        "bindings_wire.rs",
    );
    generate_bindings_generic("src-cpp/extras.h".into(), "bindings-extra.rs");
}

fn compile_c_arducam() {
    let cpp_lib_dir = arducam_git_src();

    {
        let mut builder = cc::Build::new();
        builder
            .include(&cpp_lib_dir)
            .rig_arduino(true)
            .include(spi_include_dir())
            .include(wire_include_dir());

        //

        builder.file(format!("{}/ArduCAM.cpp", &cpp_lib_dir));

        builder.file(format!(
            "{}/libraries/Wire/src/Wire.cpp",
            arduino_include_root()
        ));
        builder.file("src-cpp/extras.cpp");

        let libname = "arducam_plus_plus";
        println!("cargo:rustc-link-lib=static={}", libname);
        builder.compile(&format!("lib{}.a", libname));
    }
    {
        let mut builder = cc::Build::new();
        builder
            .include(&cpp_lib_dir)
            .rig_arduino(false)
            //.include(spi_include_dir())
            .include(wire_include_dir());

        builder.file(format!(
            "{}/libraries/Wire/src/utility/twi.c",
            arduino_include_root()
        ));

        //writeln!(stderr(), "compiler {:?}", compiler.get_compiler());

        let libname = "arducam";
        println!("cargo:rustc-link-lib=static={}", libname);
        builder.compile(&format!("lib{}.a", libname));
    }
}

fn main() {
    generate_bindings_rs();

    compile_c_arducam();
}

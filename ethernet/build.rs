use arduino_build_helpers::{arduino_include_root, ArduinoBindgen, ArduinoBuilder};
use std::env;
use std::path::PathBuf;

fn spi_include_dir() -> String {
    format!("{}/libraries/SPI/src", arduino_include_root())
}

fn generate_bindings_rs() {
    let ethernet_git_src = format!("{}/vendor/Ethernet/src", env::var("HOME").unwrap());

    let wrapper_h = "src-cpp/wrapper.h";
    println!("cargo:rerun-if-changed={}", wrapper_h);
    let bindings = bindgen::Builder::default()
        .header(wrapper_h)
        .rig_arduino_uno()
        .clang_args(&[
            &format!("-I{}", ethernet_git_src),
            &format!("-I{}", spi_include_dir()),
            "-x",
            "c++",
        ])
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .ctypes_prefix("rust_arduino_runtime::workaround_cty") // the cty crate won't compile
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let bindings_out_file = out_path.join("bindings.rs");
    bindings
        .write_to_file(bindings_out_file)
        .expect("Couldn't write bindings!");
}

fn compile_c_ethernet() {
    let anp_dir = format!("{}/vendor/Ethernet/src", env!("HOME"));
    //let avr_tool_include = format!("{}/vendor/arduino-1.0.5/hardware/tools/avr/lib/avr/include", env!("HOME"));

    let mut builder = cc::Build::new();
    let spi_dir = spi_include_dir();
    builder
        .include(&anp_dir)
        .rig_arduino(true)
        .include(spi_dir)
        .compiler("avr-g++");

    //

    if false {
        builder.file("src-cpp/ethernet.cpp");
    } else {
        builder.file(format!("{}/Ethernet.cpp", &anp_dir));
    }
    builder.file(format!("{}/EthernetUdp.cpp", &anp_dir));
    builder.file(format!("{}/EthernetServer.cpp", &anp_dir));
    builder.file(format!("{}/socket.cpp", &anp_dir));
    builder.file(format!("{}/utility/w5100.cpp", &anp_dir));
    builder.file(format!("{}/SPI.cpp", spi_include_dir()));

    //writeln!(stderr(), "compiler {:?}", compiler.get_compiler());

    println!("cargo:rustc-link-lib=static=ethernet");
    builder.compile("libethernet.a");
}

fn main() {
    generate_bindings_rs();

    compile_c_ethernet();
}

extern crate bindgen;

use arduino_build_helpers::ArduinoBuilder;
use regex::RegexBuilder;
use std::env;
use std::error::Error;
use std::fmt::Write as Write2;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=wrapper.h");
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_args(&[
            "-I/usr/share/arduino/hardware/arduino/avr/cores/arduino/",
            "-I/usr/share/arduino/hardware/arduino/avr/variants/standard/",
            "-I/usr/avr/include",
            "-D__COMPILING_AVR_LIBC__",
            "-DF_CPU=16000000L",
            "-x",
            "c++",
            "-mmcu=atmega328p",
        ])
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .use_core() // because no_std
        .ctypes_prefix("cty")
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let bindings_out_file = out_path.join("bindings.rs");
    bindings
        .write_to_file(bindings_out_file)
        .expect("Couldn't write bindings!");

    let mut code = Vec::new();
    let sink = Box::new(&mut code);
    bindings.write(sink)?;
    let code = std::str::from_utf8(code.as_slice()).unwrap();

    {
        let gen1 = enum1(code)?;
        let gen2 = enum2(code)?;
        let out_path = out_path.join("bindings2.rs");
        //let out_path_str = out_path.to_str()?;
        let mut file = File::create(&out_path)?;
        write!(file, "{}{}", gen1, gen2)?;
    }

    //writeln!(stderr(), "created {}", bindings_out_file.to_str().unwrap());

    //

    //compile_arduino_core("/usr/share/arduino/hardware/arduino/avr/cores/arduino/");

    //

    {
        let anp_dir = format!("{}/vendor/Adafruit_NeoPixel", env!("HOME"));
        //let avr_tool_include = format!("{}/vendor/arduino-1.0.5/hardware/tools/avr/lib/avr/include", env!("HOME"));

        let mut builder = cc::Build::new();
        builder.include(anp_dir).rig_arduino(true);
        builder.file("src-cpp/neopixel.cpp");

        let basename = "neopixel";
        println!("cargo:rustc-link-lib=static={}", basename);
        builder.compile(&format!("lib{}.a", basename));
    }

    Ok(())
}

fn enum1(bindings_code: &str) -> Result<String, Box<dyn Error>> {
    let re = RegexBuilder::new(r"^\s*pub const (NEO_[RGBW]{3,4}): \S+ = (\d+);")
        .multi_line(true)
        .build()?;
    let mut rval = String::new();
    let iter = re.captures_iter(bindings_code);

    writeln!(rval, "pub enum NeoPixelColorOrder {{")?;
    for rematch in iter {
        let name = rematch
            .get(1)
            .expect("group 1 missing from NEO_ match")
            .as_str();
        let val = rematch
            .get(2)
            .expect("group 2 missing from NEO_ match")
            .as_str();
        let val = str::parse::<isize>(val)?;

        writeln!(rval, "    {} = {},", name, val)?;
    }
    writeln!(rval, "}}")?;

    Ok(rval)
}

fn enum2(bindings_code: &str) -> Result<String, Box<dyn Error>> {
    let re = RegexBuilder::new(r"^\s*pub const (NEO_KHZ\w+): \S+ = (\d+);")
        .multi_line(true)
        .build()?;
    let mut rval = String::new();
    let iter = re.captures_iter(bindings_code);

    writeln!(rval, "pub enum NeoPixelFrequency {{")?;
    for rematch in iter {
        let name = rematch
            .get(1)
            .expect("group 1 missing from NEO_ match")
            .as_str();
        let val = rematch
            .get(2)
            .expect("group 2 missing from NEO_ match")
            .as_str();
        let val = str::parse::<isize>(val)?;

        writeln!(rval, "    {} = {},", name, val)?;
    }
    writeln!(rval, "}}")?;

    Ok(rval)
}

// https://github.com/japaric-archived/photon-quickstart/issues/16

/*

execve("/usr/bin/avr-g++", ["/usr/bin/avr-g++", "-c", "-g", "-Os", "-w", "-std=gnu++11",
"-fpermissive", "-fno-exceptions", "-ffunction-sections", "-fdata-sections", "-fno-threadsafe-statics",
"-Wno-error=narrowing", "-flto", "-w", "-x", "c++", "-E", "-CC",
"-mmcu=atmega328p", "-DF_CPU=16000000L", "-DARDUINO=10807", "-DARDUINO_AVR_UNO", "-DARDUINO_ARCH_AVR",
"-I/usr/share/arduino/hardware/arduino/avr/cores/arduino",
"-I/usr/share/arduino/hardware/arduino/avr/variants/standard",
"/tmp/arduino_build_966100/sketch/sketch_jan13a.ino.cpp",
"-o", "/dev/null"], 0xc00033c000 /* 56 vars */ <unfinished ...>

 */

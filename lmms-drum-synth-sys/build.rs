extern crate cc;
extern crate bindgen;

use std::path::PathBuf;
use std::env;

fn main() {
    cc::Build::new()
        .cpp(true)
        .file("src/DrumSynth.cpp")
        .compile("drumsynth");

    let bindings = bindgen::Builder::default()
        .header("src/DrumSynth.hpp")
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

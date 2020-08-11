extern crate cbindgen;

use std::env;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    let cfg = cbindgen::Config::from_file("cbindgen.toml")
        .expect("cannot open config file");

    cbindgen::Builder::new()
        .with_crate(crate_dir)
        .with_config(cfg)
        .generate()
        .expect("cannot generate wrapper.h")
        .write_to_file("wrapper.h");
}
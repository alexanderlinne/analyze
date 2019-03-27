use std::env;
use std::io::Write;
use std::fs::File;
use std::path::PathBuf;

fn main() {
    let out_dir = env::var("OUT_DIR")
        .expect("OUT_DIR environment variable missing!");
    File::create(PathBuf::from(out_dir.clone()).join("ld.version"))
        .unwrap()
        .write_all(include_bytes!("ld.version"))
        .unwrap();
    println!{"cargo:rustc-link-search=native={}", out_dir};
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=ld.version");
}

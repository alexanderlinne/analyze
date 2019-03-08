use std::env;
use std::process::{Command, Stdio};

fn main() {
    let out_dir = env::var("OUT_DIR")
        .expect("OUT_DIR environment variable missing!");

    println!{"cargo:rustc-link-lib=static=preload"};
    println!{"cargo:rustc-link-search=native={}", out_dir};

    let cmake_source = env::current_dir()
        .expect("Working directory does not exist or isn't accessible!")
        .join("libc_preload/");
    let command = format!{"cd {} && cmake {} && make -j",
        out_dir, cmake_source.to_str().expect("to_str() failed!")};
    let output = Command::new("/bin/bash")
        .args(&["-c", command.as_str()])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .expect("Failed to build libc_preload!");

    let exit_code = output.status.code()
        .expect("While building libc_preload, CMake was terminated by a signal!");
    std::process::exit(exit_code);
}

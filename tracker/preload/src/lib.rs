#[cfg(test)]
mod tests {
    use std::env;
    use std::process::{Command, Stdio};

    #[test]
    fn test_libc_preload() {
        let out_dir = env::var("OUT_DIR")
            .expect("OUT_DIR environment variable missing!");
        let filepath = format!{"{}/test_libc_preload", out_dir};
        let output = Command::new(filepath)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()
            .expect("test_libc_preload couldn't be executed!");
        assert!{output.status.success()};
    }
}

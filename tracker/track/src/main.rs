use std::env;
use std::io;
use std::io::{BufRead, BufReader, Write};
use std::fs;
use std::fs::{File};
use std::path::{PathBuf};
use std::process::{Command, Stdio};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "track", version = "0.1.0")]
struct Config {
    #[structopt(short = "o", long = "output", default_value = "build.json")]
    output_filepath: String,

    #[structopt(short = "t", long = "tempdir", default_value = ".build_info")]
    temp_directory: String,

    #[structopt(short = "b", long = "build", required = true)]
    build_command: String
}

struct TemporaryDirectory {
    path: PathBuf,
}

impl TemporaryDirectory {
    fn create(path: PathBuf) -> TemporaryDirectory {
        if path.exists() {
            panic!{"Temporary directory already exists!"};
        }
        fs::create_dir_all(&path)
            .expect("Couldn't create temporary directory!");
        TemporaryDirectory {
            path: path
        }
    }
}

impl Drop for TemporaryDirectory {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.path)
            .expect("Couldn't remove temporary directory!");
    }
}

fn execute_build_command(config: &Config)
    -> io::Result<()>
{
    let temp_directory = fs::canonicalize(&config.temp_directory)?;
    let working_dir = env::current_dir()?.join("libpreload.so");
    let output = Command::new("/bin/bash")
        .env("LD_PRELOAD", working_dir)
        .env("TRACKER_OUTPUT_PATH",
            temp_directory.to_str().expect("String conversion failed!"))
        .args(&["-c", config.build_command.as_str()])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .expect("Build command could not be executed");
    if !output.status.success() {
        panic!{"Build failed!"};
    }
    Ok(())
}

fn collect_logfiles(directory: &str)
    -> io::Result<Vec<serde_json::Value>>
{
    let mut result = vec![];
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let file = File::open(entry.path())?;
        for line in BufReader::new(file).lines() {
            let line = line?;
            result.push(serde_json::from_str::<serde_json::Value>(&line)?);
        }
    }
    Ok(result)
}

fn create_log(config: &Config)
    -> io::Result<()>
{
    let json = collect_logfiles(config.temp_directory.as_str())?;
    let mut output_file = File::create(&config.output_filepath)?;
    output_file.write_all(serde_json::to_string(&json)?.as_bytes())?;
    Ok(())
}

fn main()
    -> io::Result<()>
{
    let config = Config::from_args();
    let _temp_directory = TemporaryDirectory::create(
        PathBuf::from(&config.temp_directory));
    execute_build_command(&config)?;
    create_log(&config)?;
    Ok(())
}

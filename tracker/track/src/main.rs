#[macro_use]
extern crate error_chain;

use std::fs;
use std::path::{PathBuf};
use structopt::StructOpt;

pub mod error {
    error_chain!{}
}

pub mod build;
mod util;

use crate::error::*;

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

fn run(config: &Config)
    -> Result<()>
{
    let _temp_directory = util::TemporaryDirectory::create(
        PathBuf::from(&config.temp_directory));
    let build = build::Build::from_command(
        config.build_command.as_str(),
        config.temp_directory.as_str())
        .chain_err(|| "Tracking of the build process resulted in incosistent data!")?;
    let contents = serde_json::to_string_pretty(&build)
        .expect("JSON serialization failed unexpectedly!");
    fs::write(&config.output_filepath, contents.as_bytes())
        .chain_err(|| format!("Failed to write to file \"{}\"!", config.output_filepath))?;
    Ok(())
}

fn main()
{
    let config = Config::from_args();
    if let Err(ref e) = run(&config) {
        println!("\n~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~\n");
        println!("Failed to track build:");
        println!("error: {}", e);
        for e in e.iter().skip(1) {
            println!("caused by: {}", e);
        }

        if let Some(backtrace) = e.backtrace() {
            println!("backtrace: {:?}", backtrace);
        }

        std::process::exit(1);
    }
}

extern crate failure;

use analyze_base::build;
use failure::Error;
use std::fs;
use structopt::StructOpt;
use tempfile::tempdir;

#[derive(StructOpt, Debug)]
#[structopt(name = "track", version = "0.1.0")]
struct Config {
    #[structopt(short = "o", long = "output", default_value = "build.json")]
    output_filepath: String,

    #[structopt(short = "b", long = "build", required = true)]
    build_command: String
}

fn run(config: &Config)
    -> Result<(), Error>
{
    let dir = tempdir().unwrap();
    let build = build::Build::from_command(
        config.build_command.as_str(),
        dir.path())?;
    let contents = serde_json::to_string_pretty(&build)
        .expect("JSON serialization failed unexpectedly!");
    fs::write(&config.output_filepath, contents.as_bytes())?;
    dir.close().unwrap();
    Ok(())
}

fn main()
{
    let config = Config::from_args();
    if let Err(ref e) = run(&config) {
        println!("\n~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~\n");
        println!("Failed to track build:");
        println!("error: {}", e);
        for e in e.iter_causes() {
            println!("caused by: {}", e);
        }

        println!("backtrace: {:?}", e.backtrace());

        std::process::exit(1);
    }
}

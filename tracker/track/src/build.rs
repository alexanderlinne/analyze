use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::process::{Command, Stdio};

#[derive(Serialize, Deserialize)]
pub struct LogEntry {
    pub r#type: String,
    pub timestamp: usize,
    pub data: serde_json::Value,
}

#[derive(Serialize, Deserialize)]
pub struct Process {
    pub pid: usize,
    pub ppid: usize,
    pub argv: Vec<String>,
    pub log: Vec<LogEntry>,
}

#[derive(Serialize, Deserialize)]
pub struct Build {
    pub command: String,
    pub processes: Vec<Process>,
}

use super::error::*;

impl Build {
    pub fn new(command: &str, processes: Vec<Process>)
        -> Result<Build>
    {
        Ok(Build {
            command: command.to_string(),
            processes: processes,
        })
    }

    pub fn from_command(command: &str, temp_directory: &str)
        -> Result<Build>
    {
        let processes = Build::execute_command(command, temp_directory)?;
        Build::new(command, processes)
    }

    fn collect_processes(directory: &str)
        -> Result<Vec<Process>>
    {
        let mut processes = vec![];
        let dir_iter = fs::read_dir(directory)
            .chain_err(|| format!("Could not open directory \"{}\"!", directory))?;
        for entry in dir_iter {
            let entry = entry.chain_err(|| format!{"Iterating directory \"{}\" failed!", directory})?;
            let contents = fs::read_to_string(entry.path())
                .chain_err(|| format!("Could not read file \"{}\"!", entry.path().to_str().unwrap()))?;
            let contents : Process = serde_json::from_str(&contents)
                .chain_err(|| format!("Could not parse JSON value \"{}\"!", contents))?;
            processes.push(contents);
        }
        Ok(processes)
    }

    fn execute_command(command: &str, temp_directory: &str)
        -> Result<Vec<Process>>
    {
        let temp_directory = fs::canonicalize(temp_directory)
            .chain_err(|| format!("Failed to canonicalize path \"{}\"!", temp_directory))?;
        let current_dir = env::current_dir()
            .chain_err(|| "Failed to get the current working directory!")?;
        let working_dir = current_dir.join("libpreload.so");
        let output = Command::new("/bin/bash")
            .env("LD_PRELOAD", working_dir)
            .env("TRACKER_OUTPUT_PATH", &temp_directory)
            .args(&["-c", command])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()
            .expect("Build command could not be executed");
        if !output.status.success() {
            if let Some(code) = output.status.code() {
                Err(Error::from_kind(ErrorKind::Msg(format!("Build exited with status \"{}\"!", code))))
            } else {
                Err(Error::from_kind(ErrorKind::Msg("Build exited due to signal!".to_string())))
            }
        } else {
            Ok(Build::collect_processes(temp_directory.to_str().unwrap())?)
        }
    }
}

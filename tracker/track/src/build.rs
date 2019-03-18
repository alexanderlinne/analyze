use serde::{Deserialize, Serialize};
use std::collections::{HashMap};
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
        Build::verify_integrity(&processes)?;
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

    /**
     * Checks that the pids and ppids of all processes form a single connected
     * component and no pid is duplicate.
     */
    fn verify_integrity(processes: &Vec<Process>)
        -> Result<()>
    {
        let pids : Vec<usize> = processes.iter()
            .map(|process| process.pid)
            .collect();
        let unique_pids : HashMap<usize, ()> = pids.iter()
            .map(|pid| (*pid, ()))
            .collect();
        if pids.len() != unique_pids.len() {
            return Err(Error::from_kind(ErrorKind::Msg(
                "Duplicate pid found!".to_string())));
        }
        let parentless_count = processes.iter()
            .map(|process| process.ppid)
            .map(|ppid| if pids.contains(&ppid) { 0 } else { 1 })
            .fold(0, |a, b| a + b);
        if parentless_count > 1 {
            return Err(Error::from_kind(ErrorKind::Msg(
                "Multiple connected components found! \
                Possibly the data for a process is missing!".to_string())));
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn create_dummy_process(pid: usize, ppid: usize)
        -> Process
    {
        Process {
            pid: pid,
            ppid: ppid,
            argv: vec![],
            log: vec![],
        }
    }

    #[test]
    fn test_build_integrity_empty_vec() {
        let processes = vec![];
        assert!(Build::new("", processes).is_ok());
    }

    #[test]
    fn test_build_integriy_single_process() {
        let processes = vec![
            create_dummy_process(1, 0),
        ];
        assert!(Build::new("", processes).is_ok());
    }

    #[test]
    fn test_build_integrity_success() {
        let processes = vec![
            create_dummy_process(1, 0),
            create_dummy_process(2, 1),
            create_dummy_process(3, 1),
            create_dummy_process(4, 2),
        ];
        assert!(Build::new("", processes).is_ok());
    }

    #[test]
    fn test_build_integrity_multiple_connected_components_failure() {
        let processes = vec![
            create_dummy_process(1, 0),
            create_dummy_process(2, 1),
            create_dummy_process(3, 1),
            create_dummy_process(5, 4),
        ];
        assert!(Build::new("", processes).is_err());
    }

    #[test]
    fn test_build_integrity_duplicate_pid() {
        let processes = vec![
            create_dummy_process(1, 0),
            create_dummy_process(2, 1),
            create_dummy_process(3, 1),
            create_dummy_process(1, 3),
        ];
        assert!(Build::new("", processes).is_err());
    }
}

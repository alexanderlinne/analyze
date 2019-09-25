use failure::Error;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap};
use std::env;
use std::path::{Path};
use std::time::{SystemTime, UNIX_EPOCH};
use std::fs;
use std::process::{Command, ExitStatus, Stdio};
use libc;

#[derive(Serialize, Deserialize)]
pub struct LibcExec {
    function_name: String,
    filename: String,
    argv: Vec<String>,
    envp: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct LibcExit {
    function_name: String,
    status: usize,
}

#[derive(Serialize, Deserialize)]
pub enum EventData {
    LdPreloadLoaded(),
    LdPreloadUnloaded(),
    LibcExec(LibcExec),
    LibcExit(LibcExit),
}

#[derive(Serialize, Deserialize)]
pub struct Event {
    pub tid: usize,
    pub timestamp: usize,
    pub data: EventData,
}

impl Event {
    pub fn from(data: EventData) -> Event {
        let since_the_epoch = SystemTime::now().duration_since(UNIX_EPOCH)
            .expect("SystemTime::now() returned a time point earlier than UNIX_EPOCH!");
        let timestamp = since_the_epoch.as_secs() as usize * 1_000_000 +
            since_the_epoch.subsec_nanos() as usize / 1_000;
        Event {
            tid: unsafe { libc::syscall(libc::SYS_gettid) } as usize,
            timestamp,
            data,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Process {
    pub pid: usize,
    pub ppid: usize,
    pub argv: Vec<String>,
    pub envp: Vec<String>,
    pub working_dir: String,
    pub events: Vec<Event>,
}

#[derive(Serialize, Deserialize)]
pub struct Build {
    pub command: String,
    pub processes: Vec<Process>,
}

#[derive(Debug, Fail)]
enum BuildExecutionError {
    #[fail(display = "build exited with status: {}", code)]
    BuildExitedWithCode {
        code: i32,
    },
    #[fail(display = "build exited due to a signal")]
    BuildExitedWithSignal,
    #[fail(display = "directory '{}' has no parent directory", directory)]
    NoParentDirectory {
        directory: String,
    },
    #[fail(display = "recorded data contains duplicated pid")]
    DuplicatePid,
    #[fail(display = "a process is missing from the recorded data")]
    MissingProcess,
}

impl Build {
    pub fn new(command: &str, processes: Vec<Process>)
        -> Result<Build, Error>
    {
        Build::verify_integrity(&processes)?;
        Ok(Build {
            command: command.to_string(),
            processes: processes,
        })
    }

    pub fn from_command(command: &str, tempdir: &Path)
        -> Result<Build, Error>
    {
        let processes = Build::execute_command(command, tempdir)?;
        Build::new(command, processes)
    }

    fn execute_command(command: &str, tempdir: &Path)
        -> Result<Vec<Process>, Error>
    {
        let status = Build::execute_with_tracker(command, &tempdir)?;
        if !status.success() {
            if let Some(code) = status.code() {
                Err(BuildExecutionError::BuildExitedWithCode { code })?
            } else {
                Err(BuildExecutionError::BuildExitedWithSignal)?
            }
        } else {
            Ok(Build::collect_processes(tempdir.to_str().unwrap())?)
        }
    }

    fn execute_with_tracker(command: &str, tempdir: &Path)
        -> Result<ExitStatus, Error>
    {
        let executable_dir = fs::canonicalize(env::current_exe()?)?;
        let preload_lib = executable_dir.parent()
            .map(|p| p.join("libpreload.so"))
            .ok_or(BuildExecutionError::NoParentDirectory {
                directory: executable_dir.into_os_string().into_string().unwrap()
            })?;
        Ok(Command::new("/bin/bash")
            .env("LD_PRELOAD", preload_lib)
            .env("TRACKER_OUTPUT_PATH", &tempdir)
            .args(&["-c", command])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()
            .expect("Build command could not be executed")
            .status)
    }

    fn collect_processes(directory: &str)
        -> Result<Vec<Process>, Error>
    {
        let mut processes = vec![];
        let dir_iter = fs::read_dir(directory)?;
        for entry in dir_iter {
            let contents : Process = serde_json::from_str(
                &fs::read_to_string(entry?.path())?)?;
            processes.push(contents);
        }
        Ok(processes)
    }


    fn verify_integrity(processes: &Vec<Process>)
        -> Result<(), Error>
    {
        let pids : Vec<usize> = processes.iter()
            .map(|process| process.pid)
            .collect();
        let unique_pids : HashMap<usize, ()> = pids.iter()
            .map(|pid| (*pid, ()))
            .collect();
        if pids.len() != unique_pids.len() {
            return Err(BuildExecutionError::DuplicatePid)?;
        }
        let parentless_count = processes.iter()
            .map(|process| process.ppid)
            .map(|ppid| if pids.contains(&ppid) { 0 } else { 1 })
            .fold(0, |a, b| a + b);
        if parentless_count > 1 {
            return Err(BuildExecutionError::MissingProcess)?;
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
            envp: vec![],
            working_dir: String::from(""),
            events: vec![],
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

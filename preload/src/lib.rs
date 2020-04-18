use analyze_base::build::{Event, EventData, Process};
use libc::{c_char, c_int};
use std::env;
use std::ffi::CStr;
use std::fs;
use std::slice;
use std::sync::Mutex;

struct Logfile {
    mutex: Mutex<()>,
    output_path: String,
    process: Process,
}

impl Logfile {
    pub fn new(argv: &[*const c_char], output_path: &str) -> Logfile {
        Logfile {
            mutex: Mutex::new(()),
            output_path: String::from(output_path),
            process: Process {
                pid: unsafe { libc::getpid() } as usize,
                ppid: unsafe { libc::getppid() } as usize,
                argv: argv
                    .iter()
                    .map(|arg| {
                        unsafe { CStr::from_ptr(*arg) }
                            .to_string_lossy()
                            .to_string()
                    })
                    .collect(),
                envp: env::vars().map(|(k, v)| format!("{}={}", k, v)).collect(),
                working_dir: env::current_dir().unwrap().to_string_lossy().to_string(),
                events: vec![],
            },
        }
    }

    pub fn add(&mut self, event: Event) {
        let _ = self.mutex.lock().unwrap();
        self.process.events.push(event);
    }
}

impl Drop for Logfile {
    fn drop(&mut self) {
        let filepath = format!("{}/{}.json", self.output_path, self.process.pid);
        let contents =
            serde_json::to_string(&self.process).expect("JSON serialization failed unexpectedly!");
        fs::write(filepath, contents.as_bytes()).expect("Couldn't write logfile!");
    }
}

static mut LOGFILE: *mut Logfile = std::ptr::null_mut();

fn add_event(event: Event) {
    if unsafe { LOGFILE } != std::ptr::null_mut() {
        unsafe { &mut *LOGFILE }.add(event);
    }
}

#[link_section = ".init_array"]
pub static LD_CONSTRUCTOR: extern "C" fn(c_int, *const *const c_char) = constructor;
extern "C" fn constructor(argc: c_int, argv: *const *const c_char) {
    match env::var("TRACKER_OUTPUT_PATH") {
        Ok(var) => unsafe {
            let argv = slice::from_raw_parts(argv, argc as usize);
            LOGFILE = Box::into_raw(Box::new(Logfile::new(argv, var.as_str())))
        },
        Err(_) => (),
    };
    add_event(Event::from(EventData::LdPreloadLoaded()));
}

#[link_section = ".fini_array"]
pub static LD_DESTRUCTOR: extern "C" fn() = destructor;
extern "C" fn destructor() {
    add_event(Event::from(EventData::LdPreloadUnloaded()));
    if unsafe { LOGFILE } != std::ptr::null_mut() {
        let _ = unsafe { Box::from_raw(LOGFILE) };
        unsafe {
            LOGFILE = std::ptr::null_mut();
        }
    }
}

#[cfg(test)]
mod tests {}

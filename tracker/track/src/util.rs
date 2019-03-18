use std::fs;
use std::path::{PathBuf};

pub struct TemporaryDirectory {
    path: PathBuf,
}

impl TemporaryDirectory {
    pub fn create(path: PathBuf) -> TemporaryDirectory {
        if path.exists() {
            panic!("Temporary directory already exists!");
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

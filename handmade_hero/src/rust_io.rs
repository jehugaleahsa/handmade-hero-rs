use std::path::PathBuf;

use handmade_hero_interface::application_error::{ApplicationError, Result};
use handmade_hero_interface::io::Io;

#[derive(Debug)]
pub struct RustIo {
    root: PathBuf,
}

impl RustIo {
    #[inline]
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }
}

impl Io for RustIo {
    fn read_file_contents(&self, path: &str) -> Result<Vec<u8>> {
        if path.is_empty() {
            return Err(ApplicationError::new(
                "A relative path to the file must be provided",
            ));
        }
        let mut file_path = self.root.clone();
        for part in path.split('/') {
            file_path.push(part);
        }
        std::fs::read(file_path)
            .map_err(|e| ApplicationError::wrap("Could not read the file contents", e))
    }
}

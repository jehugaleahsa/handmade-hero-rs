use crate::application_error::Result;

pub trait Io {
    /// Read the full contents of a file into memory.
    /// # Errors
    /// If the file is not found or an error occurs while reading it, an error result is returned.
    fn read_file_contents(&self, path: &str) -> Result<Vec<u8>>;
}

use std::path::PathBuf;

/// The options for the service
pub struct Options {
    root_dir: PathBuf,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            root_dir: PathBuf::from("./"),
        }
    }
}
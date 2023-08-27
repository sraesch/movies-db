use std::{net::SocketAddr, path::PathBuf};

/// The options for the service
#[derive(Debug, Clone)]
pub struct Options {
    pub root_dir: PathBuf,

    /// The address to bind the HTTP server to.
    pub http_address: SocketAddr,

    /// The path to where ffmpeg and ffprobe are located
    pub ffmpeg: PathBuf,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            root_dir: PathBuf::from("./"),
            http_address: SocketAddr::from(([127, 0, 0, 1], 3030)),
            ffmpeg: PathBuf::from("/usr/bin/"),
        }
    }
}

use std::{net::SocketAddr, path::PathBuf};

/// The options for the service
#[derive(Debug, Clone)]
pub struct Options {
    pub cache_dir: PathBuf,

    /// The address to bind the HTTP server to.
    pub http_address: SocketAddr,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            cache_dir: PathBuf::from("./"),
            http_address: SocketAddr::from(([127, 0, 0, 1], 3030)),
        }
    }
}

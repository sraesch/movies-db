use std::path::{Path, PathBuf};

use log::info;
use tokio::process::Command;

use crate::Error;

pub struct FFMpeg {
    ffmpeg_bin_path: PathBuf,
    ffprobe_bin_path: PathBuf,
}

/// creates and returns the path to the ffmpeg binary.
///
/// # Arguments
/// * `root_dir` - The path to the directory where ffmpeg is located.
fn create_ffmpeg_bin_path(ffmpeg_dir: &Path) -> PathBuf {
    let mut ffmpeg_bin_path = ffmpeg_dir.to_path_buf();
    ffmpeg_bin_path.push("ffmpeg");

    ffmpeg_bin_path
}

/// creates and returns the path to the ffprobe binary.
///
/// # Arguments
/// * `ffprobe_dir` - The path to the directory where ffprobe is located.
fn create_ffprobe_bin_path(ffprobe_dir: &Path) -> PathBuf {
    let mut ffprobe_bin_path = ffprobe_dir.to_path_buf();
    ffprobe_bin_path.push("ffprobe");

    ffprobe_bin_path
}

impl FFMpeg {
    pub async fn new(root_dir: &Path) -> Result<Self, Error> {
        let ffmpeg_bin_path = create_ffmpeg_bin_path(root_dir);
        let ffprobe_bin_path = create_ffprobe_bin_path(root_dir);

        Self::check_bin(&ffmpeg_bin_path, "ffmpeg").await?;
        Self::check_bin(&ffmpeg_bin_path, "ffprobe").await?;

        Ok(Self {
            ffmpeg_bin_path,
            ffprobe_bin_path,
        })
    }

    /// Checks either ffmpeg or ffprobe binary.
    ///
    /// # Arguments
    /// * `bin` - The path to the binary to check.
    /// * `name` - The name of the binary to check.
    async fn check_bin(bin: &Path, name: &str) -> Result<(), Error> {
        let output = Command::new(bin)
            .arg("-version")
            .output()
            .await
            .map_err(|e| {
                Error::Internal(format!(
                    "Failed to execute ffmpeg binary '{}': {}",
                    bin.display(),
                    e
                ))
            })?;

        if !output.status.success() {
            return Err(Error::Internal(format!(
                "Failed to execute ffmpeg binary '{}': {}",
                bin.display(),
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        info!(
            "{} Version Info: {}",
            name,
            String::from_utf8_lossy(&output.stdout)
        );

        Ok(())
    }
}

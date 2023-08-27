use std::path::{Path, PathBuf};

use log::{info, trace};
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
    /// Creates a new instance of ffmpeg.
    ///
    /// # Arguments
    /// * `root_dir` - The path to the directory where the ffmpeg and ffprobe binaries are located.
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

    /// Retur ns the duration of the given movie file in seconds.
    pub async fn get_movie_duration(&self, movie_file: &Path) -> Result<f64, Error> {
        trace!("get_movie_duration: movie_file={}", movie_file.display());
        let output = Command::new(&self.ffprobe_bin_path)
            .arg("-v")
            .arg("error")
            .arg("-show_entries")
            .arg("format=duration")
            .arg("-of")
            .arg("default=noprint_wrappers=1:nokey=1")
            .arg(movie_file)
            .output()
            .await
            .map_err(|e| {
                Error::Internal(format!(
                    "Failed to execute ffprobe binary '{}': {}",
                    self.ffprobe_bin_path.display(),
                    e
                ))
            })?;

        if !output.status.success() {
            return Err(Error::Internal(format!(
                "Failed to execute ffprobe binary '{}': {}",
                self.ffprobe_bin_path.display(),
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        let duration = String::from_utf8_lossy(&output.stdout)
            .trim()
            .parse::<f64>()
            .map_err(|e| {
                Error::Internal(format!(
                    "Failed to parse ffprobe output '{}': {}",
                    String::from_utf8_lossy(&output.stdout),
                    e
                ))
            })?;

        Ok(duration)
    }

    /// Creates a screenshot of the given movie file at the given timestamp.
    ///
    /// # Arguments
    /// * `movie_file` - The path to the movie file.
    /// * `timestamp` - The timestamp in seconds at which to create the screenshot.
    pub async fn create_screenshot(
        &self,
        movie_file: &Path,
        timestamp: f64,
    ) -> Result<Vec<u8>, Error> {
        // trigger ffmpeg to create a screenshot of the given movie file at the given timestamp
        // and return the screenshot data in png format
        let output = Command::new(&self.ffmpeg_bin_path)
            .arg("-ss")
            .arg(timestamp.to_string())
            .arg("-i")
            .arg(movie_file)
            .arg("-vframes")
            .arg("1")
            .arg("-q:v")
            .arg("2")
            .arg("-c:v")
            .arg("png")
            .arg("-f")
            .arg("image2pipe")
            .arg("-")
            .output()
            .await
            .map_err(|e| {
                Error::Internal(format!(
                    "Failed to execute ffmpeg binary '{}': {}",
                    self.ffmpeg_bin_path.display(),
                    e
                ))
            })?;

        if !output.status.success() {
            return Err(Error::Internal(format!(
                "Failed to execute ffmpeg binary '{}': {}",
                self.ffmpeg_bin_path.display(),
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(output.stdout)
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

        // extract first line of version info
        let output = String::from_utf8_lossy(&output.stdout);
        let output = output.lines().next().unwrap_or_default();

        info!("{} Version Info: {}", name, output);

        Ok(())
    }
}

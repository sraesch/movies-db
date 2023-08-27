use std::sync::Arc;

use log::{debug, error, info, trace};
use tokio::{
    io::AsyncWriteExt,
    sync::{mpsc, RwLock},
};

use crate::{ffmpeg::FFMpeg, MovieDataType, MovieId, MovieStorage, MoviesIndex};

/// The request to generate a preview.
#[derive(Clone, Debug)]
pub struct ScreenshotRequest {
    pub movie_id: MovieId,
    pub ext: String,
}

pub struct PreviewGenerator<I: MoviesIndex, S: MovieStorage> {
    ffmpeg: FFMpeg,
    index: Arc<RwLock<I>>,
    storage: Arc<RwLock<S>>,
    recv_preview: mpsc::UnboundedReceiver<ScreenshotRequest>,
}

impl<I: MoviesIndex, S: MovieStorage> PreviewGenerator<I, S> {
    /// Creates a new instance of the preview generator.
    ///
    /// # Arguments
    /// * `ffmpeg` - The ffmpeg instance.
    /// * `index` - The movie index.
    /// * `storage` - The movie storage.
    pub fn new(
        ffmpeg: FFMpeg,
        index: Arc<RwLock<I>>,
        storage: Arc<RwLock<S>>,
    ) -> (Self, mpsc::UnboundedSender<ScreenshotRequest>) {
        let (send_preview, recv_preview) = mpsc::unbounded_channel();

        (
            Self {
                ffmpeg,
                index,
                storage,
                recv_preview,
            },
            send_preview,
        )
    }

    /// Runs the preview generator loop.
    pub async fn run(&mut self) {
        while let Some(r) = self.recv_preview.recv().await {
            info!("Generating preview for movie '{}'", r.movie_id);

            let file_path = match self
                .storage
                .read()
                .await
                .get_file_path(
                    r.movie_id.clone(),
                    MovieDataType::MovieData { ext: r.ext.clone() },
                )
                .await
            {
                Err(err) => {
                    error!("Failed to get movie file path for movie '{}'", r.movie_id);
                    error!("Error: {}", err);
                    continue;
                }
                Ok(file_path) => match file_path {
                    None => {
                        error!("File paths are not supported by backend");
                        continue;
                    }
                    Some(file_path) => file_path,
                },
            };

            debug!("Movie file path: {}", file_path.display());

            // determine the total duration of the movie
            trace!("Getting movie duration...");
            let duration = match self.ffmpeg.get_movie_duration(&file_path).await {
                Err(err) => {
                    error!("Failed to get movie duration for movie '{}'", r.movie_id);
                    error!("Error: {}", err);
                    continue;
                }
                Ok(duration) => duration,
            };

            // we make the screenshot in the middle of the movie
            let time_stamp = duration / 2.0;
            let screenshot_data = match self.ffmpeg.create_screenshot(&file_path, time_stamp).await
            {
                Ok(data) => data,
                Err(err) => {
                    error!("Failed to create screenshot for movie '{}'", r.movie_id);
                    error!("Error: {}", err);
                    continue;
                }
            };

            // write screenshot data
            trace!("Write screenshot data...");
            let mut writer = match self
                .storage
                .read()
                .await
                .write_movie_data(
                    r.movie_id.clone(),
                    MovieDataType::ScreenshotData { ext: r.ext.clone() },
                )
                .await
            {
                Ok(writer) => writer,
                Err(err) => {
                    error!("Failed to write screenshot data for movie '{}'", r.movie_id);
                    error!("Error: {}", err);
                    continue;
                }
            };

            if let Err(err) = writer.write_all(&screenshot_data).await {
                error!("Failed to write screenshot data for movie '{}'", r.movie_id);
                error!("Error: {}", err);
                continue;
            }
        }

        info!("Preview generator loop stopped");
    }
}

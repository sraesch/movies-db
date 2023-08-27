use std::sync::Arc;

use log::{debug, error, info, trace};
use tokio::{
    io::AsyncWriteExt,
    sync::{mpsc, RwLock},
};

use crate::{
    ffmpeg::FFMpeg, MovieDataType, MovieId, MovieSearchQuery, MovieStorage, MoviesIndex,
    ScreenshotInfo,
};

/// The request to generate a preview.
#[derive(Clone, Debug)]
pub struct ScreenshotRequest {
    /// The id of the movie to generate the preview for.
    pub movie_id: MovieId,
    pub ext: String,
}

pub struct PreviewGenerator<I: MoviesIndex, S: MovieStorage> {
    ffmpeg: FFMpeg,
    index: Arc<RwLock<I>>,
    storage: Arc<RwLock<S>>,
    recv_preview: mpsc::UnboundedReceiver<ScreenshotRequest>,
    send_preview: mpsc::UnboundedSender<ScreenshotRequest>,
}

impl<I: MoviesIndex, S: MovieStorage> PreviewGenerator<I, S> {
    /// Creates a new instance of the preview generator.
    ///
    /// # Arguments
    /// * `ffmpeg` - The ffmpeg instance.
    /// * `index` - The movie index.
    /// * `storage` - The movie storage.
    pub fn new(ffmpeg: FFMpeg, index: Arc<RwLock<I>>, storage: Arc<RwLock<S>>) -> Self {
        let (send_preview, recv_preview) = mpsc::unbounded_channel();

        Self {
            ffmpeg,
            index,
            storage,
            recv_preview,
            send_preview: send_preview.clone(),
        }
    }

    /// Returns the sender for preview requests.
    pub fn get_preview_request_sender(&self) -> mpsc::UnboundedSender<ScreenshotRequest> {
        self.send_preview.clone()
    }

    /// Runs the preview generator loop.
    pub async fn run(&mut self) {
        self.trigger_all_missing_previews().await;

        info!("Starting preview generator loop...");

        while let Some(r) = self.recv_preview.recv().await {
            debug!("Generating preview for request '{:?}'", r);

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
                    MovieDataType::ScreenshotData {
                        ext: "png".to_owned(),
                    },
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

            // update movie index about the new screenshot
            trace!("Update movie index...");
            match self
                .index
                .write()
                .await
                .update_screenshot_info(
                    &r.movie_id,
                    ScreenshotInfo {
                        extension: "png".to_owned(),
                        mime_type: "image/png".to_owned(),
                    },
                )
                .await
            {
                Ok(_) => {}
                Err(err) => {
                    error!("Failed to update movie index for movie '{}'", r.movie_id);
                    error!("Error: {}", err);
                    continue;
                }
            }
        }

        info!("Preview generator loop stopped");
    }

    async fn trigger_all_missing_previews(&self) {
        info!("Triggering all missing previews...");
        let index = self.index.read().await;

        let preview_request_sender = self.get_preview_request_sender();

        let query: MovieSearchQuery = Default::default();
        let movie_ids = match index.search_movies(query).await {
            Ok(movie_ids) => movie_ids,
            Err(err) => {
                error!("Failed to search movies");
                error!("Error: {}", err);
                return;
            }
        };

        for movie_id in movie_ids.iter() {
            let movie = match index.get_movie(movie_id).await {
                Ok(movie) => movie,
                Err(err) => {
                    error!("Failed to get movie '{}'", movie_id);
                    error!("Error: {}", err);
                    continue;
                }
            };

            if let Some(movie_file_info) = movie.movie_file_info {
                if movie.screenshot_file_info.is_none() {
                    info!("Movie '{}' is missing a preview", movie_id);
                    if let Err(err) = preview_request_sender.send(ScreenshotRequest {
                        movie_id: movie_id.clone(),
                        ext: movie_file_info.extension,
                    }) {
                        error!("Failed to send preview request for movie '{}'", movie_id);
                        error!("Error: {}", err);
                    }
                }
            }
        }
    }
}

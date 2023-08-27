use crate::{
    Error, Movie, MovieDataType, MovieId, MovieSearchQuery, MovieStorage, MoviesIndex,
    ReadResource, ScreenshotInfo,
};

use actix_multipart::Multipart;
use actix_web::body::SizedStream;
use actix_web::http::header;
use actix_web::HttpResponse;
use actix_web::{web, Responder, Result};
use futures::{StreamExt, TryStreamExt};
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::sync::{mpsc, RwLock};

use tokio_util::io::ReaderStream;

use super::preview_generator::ScreenshotRequest;

pub struct ServiceHandler<I, S>
where
    I: MoviesIndex,
    S: MovieStorage,
{
    index: Arc<RwLock<I>>,
    storage: Arc<RwLock<S>>,
    preview_requests: mpsc::UnboundedSender<ScreenshotRequest>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MovieListEntry {
    id: MovieId,
    title: String,
}

impl<I, S> ServiceHandler<I, S>
where
    I: MoviesIndex,
    S: MovieStorage,
{
    /// Creates a new instance of the service handler.
    ///
    /// # Arguments
    /// * `index` - The movies index.
    /// * `storage` - The movie storage.
    /// * `preview_requests` - The channel for sending preview requests.
    pub async fn new(
        index: Arc<RwLock<I>>,
        storage: Arc<RwLock<S>>,
        preview_requests: mpsc::UnboundedSender<ScreenshotRequest>,
    ) -> Result<Self, Error> {
        Ok(Self {
            index,
            storage,
            preview_requests,
        })
    }

    /// Handles the request to add a new movie.
    ///
    /// # Arguments
    /// * `movie` - The movie to add.
    pub async fn handle_add_movie(&self, movie: Movie) -> Result<impl Responder> {
        match self.index.write().await.add_movie(movie).await {
            Ok(movie_id) => match self
                .storage
                .read()
                .await
                .allocate_movie_data(movie_id.clone())
                .await
            {
                Ok(()) => Ok(movie_id),
                Err(err) => Self::handle_error(err),
            },
            Err(err) => Self::handle_error(err),
        }
    }

    /// Handles the request to get a new movie.
    ///
    /// # Arguments
    /// * `movie` - The movie to get.
    pub async fn handle_get_movie(&self, id: MovieId) -> Result<impl Responder> {
        match self.index.read().await.get_movie(&id).await {
            Ok(movie) => Ok(web::Json(movie)),
            Err(err) => Self::handle_error(err),
        }
    }

    /// Handles the request to delete a new movie.
    ///
    /// # Arguments
    /// * `movie` - The movie to get.
    pub async fn handle_delete_movie(&self, id: MovieId) -> Result<impl Responder> {
        match self.index.write().await.remove_movie(&id).await {
            Ok(()) => match self.storage.read().await.remove_movie_data(id).await {
                Ok(_) => Ok(actix_web::HttpResponse::Ok()),
                Err(err) => {
                    error!("Error deleting movie: {}", err);
                    Err(actix_web::error::ErrorInternalServerError(err))
                }
            },
            Err(err) => Self::handle_error(err),
        }
    }

    /// Handles the request to upload a movie.
    ///
    /// # Arguments
    /// * `id` - The id of the movie to upload.
    /// * `multipart` - The multipart data of the movie.
    pub async fn handle_upload_movie(
        &self,
        id: MovieId,
        mut multipart: Multipart,
    ) -> Result<impl Responder> {
        info!("Uploading movie {} ...", id);

        // iterate over multipart stream
        while let Ok(Some(mut field)) = multipart.try_next().await {
            // extract the filename
            let content_type = field.content_disposition();
            let filename: PathBuf = match content_type.get_filename() {
                Some(filename) => PathBuf::from(filename),
                None => {
                    error!("Invalid filename");
                    return Err(actix_web::error::ErrorBadRequest("Invalid filename"));
                }
            };

            // extract content type information
            let content_type: String = match field.headers().get(header::CONTENT_TYPE) {
                Some(content_type) => content_type.to_str().unwrap().to_string(),
                None => {
                    error!("Invalid content type");
                    return Err(actix_web::error::ErrorBadRequest("Invalid content type"));
                }
            };

            // check if the content type is a video
            if !content_type.starts_with("video") {
                error!("Invalid content type");
                return Err(actix_web::error::ErrorUnsupportedMediaType(
                    "Invalid content type",
                ));
            }

            info!(
                "Uploading file {:?} with mime-type {}",
                filename, content_type
            );

            // extract the extension
            let ext = match filename.extension() {
                Some(ext) => match ext.to_str() {
                    Some(ext) => ext.to_string(),
                    None => {
                        error!("Invalid extension");
                        return Err(actix_web::error::ErrorBadRequest("Invalid extension"));
                    }
                },
                None => {
                    error!("Invalid extension");
                    return Err(actix_web::error::ErrorBadRequest("Invalid extension"));
                }
            };

            debug!("Uploading file with extension: {:?}", ext);

            // open writer for storing movie data
            let mut writer = match self
                .storage
                .read()
                .await
                .write_movie_data(id.clone(), MovieDataType::MovieData { ext: ext.clone() })
                .await
            {
                Ok(writer) => writer,
                Err(err) => {
                    return Self::handle_error(err);
                }
            };

            // Field in turn is stream of *Bytes* object
            while let Some(chunk) = field.next().await {
                let data = match chunk {
                    Ok(data) => data,
                    Err(err) => {
                        error!("Error reading chunk: {}", err);
                        return Err(actix_web::error::ErrorInternalServerError(err));
                    }
                };

                match writer.write_all(&data).await {
                    Ok(_) => (),
                    Err(err) => {
                        error!("Error writing chunk: {}", err);
                        return Err(actix_web::error::ErrorInternalServerError(err));
                    }
                }
            }

            // update the movie file info
            match self
                .index
                .write()
                .await
                .update_movie_file_info(
                    &id,
                    crate::MovieFileInfo {
                        extension: ext.clone(),
                        mime_type: content_type,
                    },
                )
                .await
            {
                Ok(()) => (),
                Err(err) => {
                    error!("Error updating movie file info: {}", err);
                    return Err(actix_web::error::ErrorInternalServerError(err));
                }
            }

            if let Err(err) = self.preview_requests.send(ScreenshotRequest {
                movie_id: id.clone(),
                ext: ext.clone(),
            }) {
                error!("Error sending preview request: {}", err);
            }
        }

        info!("Uploading movie {} ... DONE", id);

        Ok(actix_web::HttpResponse::Ok())
    }

    /// Handles the request to upload a screenshot.
    ///
    /// # Arguments
    /// * `id` - The id of the movie to upload a screenshot.
    /// * `multipart` - The multipart data of the screenshot.
    pub async fn handle_upload_screenshot(
        &self,
        id: MovieId,
        mut multipart: Multipart,
    ) -> Result<impl Responder> {
        info!("Uploading screenshot {} ...", id);

        // iterate over multipart stream
        while let Ok(Some(mut field)) = multipart.try_next().await {
            // extract the filename
            let content_type = field.content_disposition();
            let filename: PathBuf = match content_type.get_filename() {
                Some(filename) => PathBuf::from(filename),
                None => {
                    error!("Invalid filename");
                    return Err(actix_web::error::ErrorBadRequest("Invalid filename"));
                }
            };

            // extract content type information
            let content_type: String = match field.headers().get(header::CONTENT_TYPE) {
                Some(content_type) => content_type.to_str().unwrap().to_string(),
                None => {
                    error!("Invalid content type");
                    return Err(actix_web::error::ErrorBadRequest("Invalid content type"));
                }
            };

            // check if the content type is an image
            if !content_type.starts_with("image") {
                error!("Invalid content type");
                return Err(actix_web::error::ErrorUnsupportedMediaType(
                    "Invalid content type",
                ));
            }

            info!(
                "Uploading screenshot {:?} with mime-type {}",
                filename, content_type
            );

            // extract the extension
            let ext = match filename.extension() {
                Some(ext) => match ext.to_str() {
                    Some(ext) => ext.to_string(),
                    None => {
                        error!("Invalid extension");
                        return Err(actix_web::error::ErrorBadRequest("Invalid extension"));
                    }
                },
                None => {
                    error!("Invalid extension");
                    return Err(actix_web::error::ErrorBadRequest("Invalid extension"));
                }
            };

            debug!("Uploading screenshot with extension: {:?}", ext);

            // open writer for storing screenshot data
            let mut writer = match self
                .storage
                .read()
                .await
                .write_movie_data(
                    id.clone(),
                    MovieDataType::ScreenshotData { ext: ext.clone() },
                )
                .await
            {
                Ok(writer) => writer,
                Err(err) => {
                    return Self::handle_error(err);
                }
            };

            // Field in turn is stream of *Bytes* object
            while let Some(chunk) = field.next().await {
                let data = match chunk {
                    Ok(data) => data,
                    Err(err) => {
                        error!("Error reading chunk: {}", err);
                        return Err(actix_web::error::ErrorInternalServerError(err));
                    }
                };

                match writer.write_all(&data).await {
                    Ok(_) => (),
                    Err(err) => {
                        error!("Error writing chunk: {}", err);
                        return Err(actix_web::error::ErrorInternalServerError(err));
                    }
                }
            }

            // update the movie screenshot info
            match self
                .index
                .write()
                .await
                .update_screenshot_info(
                    &id,
                    ScreenshotInfo {
                        extension: ext.clone(),
                        mime_type: content_type,
                    },
                )
                .await
            {
                Ok(()) => (),
                Err(err) => {
                    error!("Error updating screenshot info: {}", err);
                    return Err(actix_web::error::ErrorInternalServerError(err));
                }
            }
        }

        info!("Uploading screenshot {} ... DONE", id);

        Ok(actix_web::HttpResponse::Ok())
    }

    /// Handles the request to download a movie.
    ///
    /// # Arguments
    /// * `id` - The id of the movie to download.
    pub async fn handle_download_movie(&self, id: MovieId) -> Result<impl Responder> {
        info!("Downloading movie {} ...", id);

        // get the movie file info, needed for requesting the movie data
        let movie_file_info = match self.index.read().await.get_movie(&id).await {
            Ok(movie) => match movie.movie_file_info {
                Some(movie_file_info) => movie_file_info,
                None => {
                    error!("Movie {} has no movie file info", id);
                    return Err(actix_web::error::ErrorConflict(format!(
                        "Movie {} is not yet ready",
                        id
                    )));
                }
            },
            Err(err) => {
                error!("Error getting movie info: {}", err);
                return Self::handle_error(err);
            }
        };

        // create reader onto the movie data
        let movie_data = match self
            .storage
            .read()
            .await
            .read_movie_data(
                id,
                MovieDataType::MovieData {
                    ext: movie_file_info.extension.clone(),
                },
            )
            .await
        {
            Ok(movie_data) => movie_data,
            Err(err) => {
                error!("Error reading movie data: {}", err);
                return Self::handle_error(err);
            }
        };

        // create response
        let length = movie_data.get_size().await as u64;
        let reader_stream = ReaderStream::new(movie_data);
        let sized_stream = SizedStream::new(length, reader_stream);

        HttpResponse::Ok()
            .content_type(movie_file_info.mime_type)
            .message_body(sized_stream)
    }

    /// Handles the request to download a screenshot.
    ///
    /// # Arguments
    /// * `id` - The id of the movie whose screenshot will be downloaded.
    pub async fn handle_download_screenshot(&self, id: MovieId) -> Result<impl Responder> {
        info!("Downloading screenshot {} ...", id);

        // get the movie screenshot info, needed for requesting the data
        let screenshot_info = match self.index.read().await.get_movie(&id).await {
            Ok(movie) => match movie.screenshot_file_info {
                Some(screenshot_info) => screenshot_info,
                None => {
                    error!("Movie {} has no screenshot info", id);
                    return Err(actix_web::error::ErrorConflict(format!(
                        "Movie {} is not yet ready",
                        id
                    )));
                }
            },
            Err(err) => {
                error!("Error getting screenshot info: {}", err);
                return Self::handle_error(err);
            }
        };

        // create reader onto the screenshot data
        let screenshot_data = match self
            .storage
            .read()
            .await
            .read_movie_data(
                id,
                MovieDataType::ScreenshotData {
                    ext: screenshot_info.extension.clone(),
                },
            )
            .await
        {
            Ok(screenshot_data) => screenshot_data,
            Err(err) => {
                error!("Error reading screenshot data: {}", err);
                return Self::handle_error(err);
            }
        };

        // create response
        let length = screenshot_data.get_size().await as u64;
        let reader_stream = ReaderStream::new(screenshot_data);
        let sized_stream = SizedStream::new(length, reader_stream);

        HttpResponse::Ok()
            .content_type(screenshot_info.mime_type)
            .message_body(sized_stream)
    }

    /// Handles the request to show the list of all movies.
    pub async fn handle_search_movies(&self, query: MovieSearchQuery) -> Result<impl Responder> {
        let movie_ids = match self.index.read().await.search_movies(query).await {
            Ok(movie_ids) => movie_ids,
            Err(err) => {
                error!("Error searching: {}", err);
                return Self::handle_error(err);
            }
        };

        let mut movies: Vec<MovieListEntry> = Vec::with_capacity(movie_ids.len());
        for movie_id in movie_ids.iter() {
            let movie = self.index.read().await.get_movie(movie_id).await.unwrap();
            movies.push(MovieListEntry {
                id: movie_id.clone(),
                title: movie.movie.title,
            });
        }

        Ok(web::Json(movies))
    }

    /// Handles the given error by translating it into an actix-web error response.
    ///
    /// # Arguments
    /// * `err` - The error to handle.
    fn handle_error<T: Responder>(err: Error) -> Result<T> {
        match err {
            Error::InvalidArgument(e) => {
                error!("Invalid argument: {}", e);
                Err(actix_web::error::ErrorBadRequest(e))
            }
            Error::NotFound(e) => {
                error!("Not found: {}", e);
                Err(actix_web::error::ErrorNotFound(e))
            }
            _ => {
                error!("Internal error: {}", err);
                Err(actix_web::error::ErrorInternalServerError(err))
            }
        }
    }
}
